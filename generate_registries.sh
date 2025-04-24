#!/bin/bash

# https://minecraft.wiki/w/Minecraft_Wiki:Projects/wiki.vg_merge/Data_Generators

base_dir=target/registries
server_download_url=https://piston-data.mojang.com/v1/objects/4707d00eb834b446575d89a61a11b5d548d8c001/server.jar

java=java
homebrew_openjdk_java=/opt/homebrew/opt/openjdk/bin/java

if [ -f "$homebrew_openjdk_java" ] && "$homebrew_openjdk_java" -version; then
    echo "Using OpenJDK Java at $homebrew_openjdk_java"
    java="$homebrew_openjdk_java"
fi

mkdir -p $base_dir
cd $base_dir

if [ -f generated ]; then
    echo "Registries already generated"
    exit 0
fi

if [ ! -f minecraft_server.jar ]; then
    curl -fL -o minecraft_server.jar $server_download_url
fi

"$java" -DbundlerMainClass="net.minecraft.data.Main" -jar minecraft_server.jar --all
