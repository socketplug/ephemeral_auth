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
Receive a public token and a secret token from `/auth/init/<id>` where id is
the user that you wish to validate.  Set the user's blurb on plug.dj to the
public token (to be nice make sure to remember what their blurb was beforehand
so that you can set it back after authenticating).

#### Returns
```json
{
  "public_token": "<string>",
  "secret": "<string>"
}
```



### Authenticate
Post the secret token to `/auth/authenticate` as a string in a JSON body.
The server will fetch the user's blurb from the plug.dj api, and compare it
to the public_token encoded in your secret token.  The server will return
a response similar to

#### Returns 
Failed authentication:
```json
{
  "valid": "<why the authentication failed>",
  "token": null
}
```

Successful authentication:
```json
{
  "valid": "valid",
  "token": "your new authenticated token"
}
```

A valid authentication token will be valid for that id for 1 hour by default.



### Verify
To check if a token is valid when you already have an authentication token,
post the authentication token to `/auth/verify` as a string in a JSON body.

#### Returns 
```json
{
  "verify": true
}
```

with `false` being a failed verified authentication token.


---


enjoy i guess idk if i'll make this higher quality anytime soon.
i can't even load the plug dashboard, only the api so testing was fun