#!/bin/bash

echo "🛠  Installing node modules"
npm install

echo "🏗  Building minified javascript"
npm run build

# Trim node modules down to the essentials:
# we want the WASM to be loadable from our host
# so that CORS isn't triggered
cd node_modules

for ext in "svg" "png" "wasm" "css"
do
    echo "📦 Preserving $ext files for upload"
    find . -name "*.$ext" | xargs -n1 dirname | uniq | xargs -I '{}' mkdir -p ../tmp/{}
    find . -name "*.$ext" -exec mv {} ../tmp/{} \;
done

echo "🛀 Deleting the rest of the node_modules directory"
cd ..
rm -rf ./node_modules

mkdir node_modules
cd tmp
for ext in "svg" "png" "wasm" "css"
do
    echo "🗄  Repositioning $ext files"
    find . -name "*.$ext" | xargs -n1 dirname | uniq | xargs -I '{}' mkdir -p ../node_modules/{}
    find . -name "*.$ext" -exec mv {} ../node_modules/{} \;
done

cd ..
echo "🚀 Deploying"

firebase deploy

echo
echo "🛠 Re-installing node modules"
npm install

echo "🗑 Cleaning up temporary files"
rm -rf ./tmp

echo "🏁 Done!"

