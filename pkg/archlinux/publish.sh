#!/bin/bash

PKGEXT=".pkg.tar.xz" makepkg &&
repo-add ascii.db.tar.gz *.tar.xz &&
scp ascii.db* *.tar.xz ascii:www/packages/archlinux/ascii/os/x86_64
