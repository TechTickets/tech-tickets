#!/usr/bin/env bash

# generate a new key pair with no user input rs256 openssl
if [ ! -f "jwt_web.key" ]; then
  ssh-keygen -t rsa -b 4096 -m PEM -f private.key -N ''
  rm jwt_web.key.pub
  echo "Private key generated"
fi

# generate public key from private key
if [ ! -f "jwt_web.key.pub" ]; then
  openssl rsa -in private.key -pubout -outform PEM -out public.key
  echo "Public key generated"
fi
