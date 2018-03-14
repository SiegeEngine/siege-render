#!/bin/sh

BINPATH=/z/eob/game/client/VulkanSDK/1.0.68.0/x86_64/bin

for v in *.vert ; do
    "$BINPATH"/glslangValidator -V "${v}" -o "${v}".spv
done

for f in *.frag ; do
    "$BINPATH"/glslangValidator -V "${f}" -o "${f}".spv
done
