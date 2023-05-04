#!/bin/bash

for ((slot=190687712; slot>=191828695+1000000000000; slot++)); do
    ./client --slot $slot --rpc <>
done

