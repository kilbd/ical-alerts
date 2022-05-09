import {
  aws_lambda as lambda,
  aws_logs as logs,
  CfnOutput,
  CfnResource,
  Stack,
  StackProps,
} from "aws-cdk-lib";
import { Construct } from "constructs";

export class InfraStack extends Stack {
  constructor(scope: Construct, id: string, props?: StackProps) {
    super(scope, id, props);

    const alertLambda = new lambda.Function(this, "AlertHandler", {
      runtime: lambda.Runtime.PROVIDED_AL2,
      architecture: lambda.Architecture.ARM_64,
      handler: "alertHandler",
      code: lambda.Code.fromAsset("../lambdas/ics-alerts"),
      memorySize: 256,
      logRetention: logs.RetentionDays.ONE_WEEK,
    });

    const fnUrl = new CfnResource(this, "alertFnUrl", {
      type: "AWS::Lambda::Url",
      properties: {
        TargetFunctionArn: alertLambda.functionArn,
        AuthType: "NONE",
        Cors: { AllowOrigins: ["*"] },
      },
    });

    const fnUrlPermission = new CfnResource(this, "fnUrlPermission", {
      type: "AWS::Lambda::Permission",
      properties: {
        FunctionName: alertLambda.functionName,
        Principal: "*",
        Action: "lambda:InvokeFunctionUrl",
        FunctionUrlAuthType: "NONE",
      },
    });

    new CfnOutput(this, "funcUrl", {
      value: `${fnUrl.getAtt("FunctionUrl")}`,
    });
  }
}
