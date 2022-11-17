#!/bin/bash

src=$1
dest=$2
files=`ls $src`

currentTime=`date "+%Y-%m-%d-%H:%M:%S"`

echo $files

for i in $files 
do
    cp $src\/$i ${dest}
    
done

