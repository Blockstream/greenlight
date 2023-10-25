# Get a developer certificate

In order to build with Greenlight, you need a developer certificate. These are custom
certificates that developers can bundle with their application, and that allow
registering new nodes.

You can create an account on the [Greenlight Developer Console][gdc] and download the zip file
containing the certificate.

[gdc]: https://greenlight.blockstream.com

## Storing the Certificate

The certificate is ditributed as two `x509` PEM files bundled into a zip file:

 - `client.crt`: this is the public certificate
 - `client-key.pem`: this is the private key matching the
   above certificate and is used to encrypt the transport and
   authenticate as a partner to the Scheduler.

Consider these files as secrets. You should not include them in your
code-repository but store them somewhere else instead. 
