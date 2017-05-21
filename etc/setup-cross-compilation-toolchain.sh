#!/bin/bash

set -o errexit
set -o xtrace

export PREFIX="$HOME/.local/cross"
export TARGET=x86_64-pc-elf
export PATH="$PREFIX/bin:/usr/local/bin:/usr/bin:/bin"

BUILDDIR=$HOME/.local/build
SRCDIR=$HOME/.local/src

# binutils: https://www.gnu.org/software/binutils/
echo Install binutils
read -p'-> skip? '
if [[ $REPLY != y ]]; then
   VERSION=2.28
   if ! [[ -d $SRCDIR/binutils/$VERSION/.git ]]; then
      mkdir -p $SRCDIR/binutils/$VERSION
      curl -sSf "http://ftp.gnu.org/gnu/binutils/binutils-$VERSION.tar.gz" \
         | tar --strip-components 1 -zxC $SRCDIR/binutils/$VERSION
      (cd $SRCDIR/binutils/$VERSION && git init . && git add . && git commit -m 'committed')
   else
      (cd $SRCDIR/binutils/$VERSION && git reset --hard HEAD && git clean -fdx)
   fi

   (
      mkdir -p $BUILDDIR/binutils/$VERSION
      cd $BUILDDIR/binutils/$VERSION
      $SRCDIR/binutils/$VERSION/configure \
         --target=$TARGET                 \
         --prefix="$PREFIX"               \
         --with-sysroot                   \
         --disable-nls                    \
         --disable-werror

      make
      make install
   )
fi

# gcc: https://gcc.gnu.org/mirrors.html
echo Install gcc
read -p'-> skip? '
if [[ $REPLY != y ]]; then
   VERSION=7.1.0
   if ! [[ -d $SRCDIR/gcc/$VERSION/.git ]]; then
      mkdir -p $SRCDIR/gcc/$VERSION
      curl -sSf "http://ftpmirror.gnu.org/gcc/gcc-$VERSION/gcc-$VERSION.tar.bz2" \
         | tar --strip-components 1 -jxC $SRCDIR/gcc/$VERSION
      (cd $SRCDIR/gcc/$VERSION && git init . && git add . && git commit -m 'committed')
   else
      (cd $SRCDIR/gcc/$VERSION && git reset --hard HEAD && git clean -fdx)
   fi

   (
      mkdir -p $BUILDDIR/gcc/$VERSION
      cd $BUILDDIR/gcc/$VERSION
      $SRCDIR/gcc/$VERSION/configure \
         --target=$TARGET            \
         --prefix="$PREFIX"          \
         --disable-nls               \
         --enable-languages=c,c++    \
         --without-headers

      make all-gcc
      make all-target-libgcc
      make install-gcc
      make install-target-libgcc
   )
fi

# objconv: http://www.agner.org/optimize/objconv.zip
echo Install objconv
read -p'-> skip? '
if [[ $REPLY != y ]]; then
   if ! [[ -d $SRCDIR/objconv/.git ]]; then
      mkdir -p $SRCDIR/objconv/tmp
      curl -sSf "http://www.agner.org/optimize/objconv.zip" >$SRCDIR/objconv/tmp/.zip
      unzip $SRCDIR/objconv/tmp/.zip -d $SRCDIR/objconv/tmp
      unzip $SRCDIR/objconv/tmp/source.zip -d $SRCDIR/objconv
      (cd $SRCDIR/objconv && git init . && git add . && git commit -m 'committed')
   else
      (cd $SRCDIR/objconv && git reset --hard HEAD && git clean -fdx)
   fi

   (
      mkdir -p $BUILDDIR/objconv
      cd $BUILDDIR/objconv
      g++ -o objconv -O2 $SRCDIR/objconv/*.cpp --prefix="$PREFIX"
      cp objconv $PREFIX/bin
   )
fi

# grub: git://git.savannah.gnu.org/grub.git
echo Installing grub
read -p'-> skip? '
if [[ $REPLY != y ]]; then
   VERSION=2.02
   if ! [[ -d $SRCDIR/grub/$VERSION/.git ]]; then
      mkdir -p $SRCDIR/grub/$VERSION
      curl -sSf "ftp://ftp.gnu.org/gnu/grub/grub-$VERSION.tar.gz" \
         | tar --strip-components 1 -zxC $SRCDIR/grub/$VERSION
      (cd $SRCDIR/grub/$VERSION && git init . && git add . && git commit -m 'committed')
      (cd $SRCDIR/grub/$VERSION && ./autogen.sh && git add . && git commit -m 'autogen')
   else
      (cd $SRCDIR/grub/$VERSION && git reset --hard HEAD && git clean -fdx)
   fi

   (
      mkdir -p $BUILDDIR/grub/$VERSION
      cd $BUILDDIR/grub/$VERSION
      LEX=/usr/local/opt/flex/bin/flex $SRCDIR/grub/$VERSION/configure \
         --disable-werror \
         --target=$TARGET \
         --prefix=$PREFIX

      make
      make install
   )
fi
