# sepha: socketplug ephemeral auth [![Build Status](https://travis-ci.org/socketplug/sepha.svg?branch=master)](https://travis-ci.org/socketplug/sepha)

A self contained 3rd party auth server for plug.dj.  Please note that this is
very proof of concept and some pretty shaky code (a lot of `.expects()`).

Also includes nothing for rate limiting to the plug.dj api so it can get
rate-limited.



## Config
sepha by default looks for `.sepha.json` in the current working directory.
you can also point to a config with the first command argument.

`sepha ./alternative_config.json`

Check `.sepha.json.example` for a configuration example.



## Generating Certificates
This is how to generate a private rsa key, and transform it into the format
(DER) the application uses.  We also output a text-based public key to help
anyone using the public key.

```sh
openssl genrsa -out private.pem 4096
openssl rsa -in private.pem -outform DER -out private.der
openssl rsa -in private.pem -RSAPublicKey_out -outform DER -out public.der
openssl rsa -in private.pem -pubout -out public.pem
```

**IMPORTANT:** Make sure that your keys are owned by the same user that you are
running the application as.  Then, make the keys readable and writable to only
that user.  This can be accomplished with `chmod 600 <your_files_here>`. This
is important because the application needs to use the unencrypted key to sign
and verify, but you don't want just any user to read them.  

If you are saving/storing the private.pem file somewhere else after, then you
should ideally encrypt that version.  This can be accomplished with:

```sh
openssl rsa -aes256 -in private.pem -out secure_private.pem
```

**sepha expects a `private.der` in the current working directory**



## Using the server

### Init

`/auth/init/<id>`


Initializes the authentication process for the plug.dj user whose id was
set in `<id>`.  Returns a public token, and a secret token which is encoded
with the user's id and the public token.  The secret token is valid for 5
minutes by default (and therefore also the public token).

Make sure to keep the secret a secret (what a surprise!), and set the user's
profile blurb to the public token before making an authenticate request.

**Important:** Make sure to remember what the user's blurb was before you set
it to the public token so that you can restore it after authenticating.


#### Return Schema
```json
{
  "public": "<public_token>",
  "secret": "<secret_token>"
}
```



---



### Authenticate

`/auth/authenticate`


The server will fetch the user's blurb from the plug.dj api, and compare it
to the public_token encoded in your secret token.  Use this endpoint after you
have set the the specified user id's blurb to the public token you received
from the `/auth/init/<id>` endpoint.

#### Post Schema
Expects to be posted a key `secret` with either a form or json.  This should
be the secret token received from the `/auth/init/<id>` endpoint

`application/x-www-form-urlencoded`: `secret=<secret_token>`

`application/json`:
```json
{
  "secret": "<secret_token>"
}
```


#### Return Schema

If successfully authenticated, you will receive an `"ok"` status and a token:
```json
{
  "status": "ok",
  "token": "<your_new_authenticated_token>"
}
```
A valid authentication token will be valid for that id for 1 hour by default.


If authentication failed for whatever reason, your token will be set to `null`
and `status` will be set to the failure reason.
```json
{
  "status": "<failure_reason>",
  "token": null
}
```



---



### Verify

`/auth/verify`


Used to verify if an `/auth/authenticate` endpoint token is valid.  This will
reject invalid tokens, and tokens that have expired.


#### Post Schema
Expects to be posted a key `token` with either a form or json.

`application/x-www-form-urlencoded`: `token=<auth_token>`

`application/json`:
```json
{
  "token": "<auth_token>"
}
```


#### Returns
Returns json object with boolean key `valid` set to if the token was valid.
```json
{
  "valid": true
}
```



---



enjoy i guess idk if i'll make this higher quality anytime soon.
i can't even load the plug dashboard, only the api so testing was fun
