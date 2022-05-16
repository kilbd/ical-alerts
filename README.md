# Add Alerts to Office365 ICS Feeds

I'd like to keep using [Fantastical](https://flexibits.com/fantastical) with my employer's Office365 instance, but they've blocked it from making OAuth connections. The only alternative is to create a [shared calendar](https://support.microsoft.com/en-us/office/share-your-calendar-in-outlook-on-the-web-7ecef8ae-139c-40d9-bae2-a23977ee58d5) and add the generated ICS feed link as a subscription calendar in Fantastical. Zoom links still work well, but O365 strips alerts from the feed so I was blowing through meeting times.

This project uses an AWS Lambda function as a machine-in-the-middle to inject one or more calendar alerts for every event. It attempts to be sensitive to costs - it only adds a few milliseconds on top of the time to pull in the ICS feed (O365 response generally took around a half second in my experience). It also uses the relatively new Lambda Function URLs to avoid API Gateway costs and uses ARM architecture to trim Lambda costs. AWS resources are generated using the AWS CDK.

## Usage

Lambda Function URLs generally take the form `https://<function UUID>.lambda-url.<region>.on.aws`. Unlike API Gateway, you can't have path parameters, but you can have query parameters. This function requires 3 of them:

- **`user`**: the user ID found in your O365 share URL (see below)
- **`token`**: the token secret found in your O365 share URL (see below)
- **`min`**: the number of minutes before each event to schedule an alert. Can be repeated.

The "shared calendar" feed URL you generated in Outlook should be of the form `https://outlook.office365.com/owa/calendar/<user>/<token>/calendar.ics`. You'll need to supply the user and token from this URL for the query parameters listed above.

Putting it all together, to get alerts 15 and 5 minutes before events you'll add a calendar subscription to Fantastical with a URL that looks like the following:

```http
https://<function UUID>.lambda-url.<region>.on.aws?user=verylonguserid@mycompany.com&token=blahblahblah&min=15&min=5
```

### Requirements

This project uses Amazon's CDK for configuring a Lambda function written in Rust and its Function URL. You will need:

- Node and npm for running the AWS CDK
- Rust and Cargo for building and developing the Lambda function
- Zig to help build an AWS-compatible binary
- AWS CLI for deployments from local machine

### Deployments

To deploy this stack, follow these steps:

1. Ensure you have the AWS CLI configured for your account. See the [Getting Started guide](https://docs.aws.amazon.com/cli/latest/userguide/cli-chap-getting-started.html). For macOS and Homebrew, this may look like:

```shell
$ brew install awscli
$ aws configure
```

2. Set up bootstrapping for the project (needed for uploading Rust binaries). Get your account ID from the first command below, which you will need to provide along with the region for the bootstrap command:

```shell
$ aws sts get-caller-identity
$ npx aws-cdk bootstrap aws://{AWS_ACCOUNT_ID}/us-east-1
```

3. Install `cargo-zigbuild` per the [project instructions](https://github.com/messense/cargo-zigbuild). Note you will need to add the `aarch64-unknown-linux-gnu` target via `rustup target add`.

4. Build the Rust binary:

```shell
$ cargo zigbuild --target aarch64-unknown-linux-gnu.2.26 --release
$ cp target/aarch64-unknown-linux-gnu/release/ical-alerts lambdas/ics-alerts/bootstrap
```

5. Add `node_modules` and synthesize the CloudFormation templates to test

```shell
$ npm install
$ npx aws-cdk synth
```

6. If all went well, deploy to your AWS account. Reusing this command will replace your current deployment. Note the generated Lambda Function URL in the output.

```shell
$ npx aws-cdk deploy
```

7. To tear down, use `npx aws-cdk destroy`.
