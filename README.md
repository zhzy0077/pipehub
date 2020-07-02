PipeHub
===
![Build](https://github.com/zhzy0077/pipehub/workflows/Build/badge.svg)
![MIT Licensed](https://img.shields.io/github/license/zhzy0077/pipehub)

## Get started
It's a service that pipelines web request to your WeChat Account.

Please visit https://www.pipehub.net/ and have a try.

## Availability
According to our statistics, the overall availability during 6/18 and 7/2 is **99.67%**.

The percentiles of latencies are:
- Avg: 338ms
- p50: 192ms
- p80: 386ms
- p90: 875ms

## Use cases
- Automate: [Automate](https://llamalab.com/automate/) on `Android` can redirect short messages, FCM notifications to PipeHub. Try `Notify me.flo` under `usecases` folder.

## Deploy your own server
1. Prerequisites:
    - A PostgreSQL database.
    - A GitHub OAuth App.
2. Prepare the dotenv file(replace the placeholders):
    ```bash
    pipehub_database_url=postgres://root:123456@localhost/pipehub
    pipehub_host=0.0.0.0
    pipehub_port=8080
    pipehub_domain=http://localhost:8080
    pipehub_https=false
    pipehub_log__level=INFO
    pipehub_github__client_id=${GITHUB.OAUTH_CLIENTID}
    pipehub_github__client_secret=${GITHUB.OAUTH_SECRET}
    pipehub_github__auth_url=https://github.com/login/oauth/authorize
    pipehub_github__token_url=https://github.com/login/oauth/access_token
    pipehub_github__callback_url=http://localhost:8080/callback
    ```
- Use docker image:

    At this point, you need to login your github account before pulling a docker image as of [docker pull from public GitHub Package Registry fail with “no basic auth credentials” error](https://github.community/t/docker-pull-from-public-github-package-registry-fail-with-no-basic-auth-credentials-error/16358).
    ```bash
    docker login docker.pkg.github.com -u ${github name}
    docker pull docker.pkg.github.com/zhzy0077/pipehub/pipehub:latest
    docker run --env-file ${your dotenv file} -d docker.pkg.github.com/zhzy0077/pipehub/pipehub:latest
    ```
- Build from sratch:
    ```bash
    # Clone the repository.
    git clone https://github.com/zhzy0077/pipehub
    # Build the web.
    cd web && yarn && yarn build
    # Copy web assets.
    cd .. && cp -r web/static/* server/static/
    # Copy dotenv file.
    cp ${your dotenv file} server
    # Run the server.
    cd server && cargo run
    ```

## Feedback
All kinds of feedback is welcomed. Just feel free to get in touch with me by creating an issue or emailing zhzy0077@hotmail.com.

## License
This project is open-sourced under MIT license.
