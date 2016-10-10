#!/bin/sh
set -e

export PREFIX="$HOME/.local/cross"
export TARGET=x86_64-pc-elf
export PATH="$PREFIX/bin:/usr/local/bin:/usr/bin:/bin"

BUILDDIR=$HOME/.local/build
SOURCEDIR=$HOME/.local/src

# binutils
# assumes https://www.gnu.org/software/binutils/
echo ""
echo "Installing \`binutils\`"
echo ""
  VERSION=2.27
  (cd $SOURCEDIR/binutils-$VERSION && git reset --hard HEAD && git clean -fdx)
  mkdir -p $BUILDDIR/binutils
  cd $BUILDDIR/binutils
  $SOURCEDIR/binutils-$VERSION/configure --target=$TARGET --prefix="$PREFIX" --with-sysroot --disable-nls --disable-werror
  make
  make install

# gcc
# assumes https://gcc.gnu.org/mirrors.html
echo ""
echo "Installing \`gcc\`"
echo ""
  VERSION=6.2.0
  (cd $SOURCEDIR/gcc-$VERSION && git reset --hard HEAD && git clean -fdx)
  mkdir -p $BUILDDIR/gcc
  cd $BUILDDIR/gcc
  $SOURCEDIR/gcc-$VERSION/configure --target=$TARGET --prefix="$PREFIX" --disable-nls --enable-languages=c,c++ --without-headers
  make all-gcc
  make all-target-libgcc
  make install-gcc
  make install-target-libgcc

# objconv
# assumes http://www.agner.org/optimize/objconv.zip
echo ""
echo "Installing \`objconv\`"
echo ""
  unzip $SOURCEDIR/objconv.zip -d $BUILDDIR/objconv
  cd $BUILDDIR/objconv
  unzip ./source.zip -d ./src
  g++ -o objconv -O2 src/*.cpp --prefix="$PREFIX"
  cp objconv $PREFIX/bin

# grub
# assumes `git clone --depth 1 git://git.savannah.gnu.org/grub.git`
echo ""
echo "Installing \`grub\`"
echo ""
  VERSION=2.00
  (cd $SOURCEDIR/grub && git reset --hard HEAD && git clean -fdx)
  (cd $SOURCEDIR/grub && ./autogen.sh)
  mkdir -p $BUILDDIR/grub
  cd $BUILDDIR/grub
  LEX=/usr/local/opt/flex/bin/flex $SOURCEDIR/grub-$VERSION/configure --disable-werror --target=$TARGET --prefix=$PREFIX
  make
  make install
