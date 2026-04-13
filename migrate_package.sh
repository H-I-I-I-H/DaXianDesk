#!/bin/bash
set -e

echo ">>> Step 1: 创建新目录并移动 .kt 文件"
mkdir -p flutter/android/app/src/main/kotlin/com/daxian/dev
mv flutter/android/app/src/main/kotlin/com/xiaohao/helloworld/*.kt \
   flutter/android/app/src/main/kotlin/com/daxian/dev/
rm -rf flutter/android/app/src/main/kotlin/com/xiaohao

echo ">>> Step 2: 替换所有 .kt 文件中的包名"
find flutter/android/app/src/main/kotlin/com/daxian/dev -name "*.kt" \
  -exec sed -i 's/com\.xiaohao\.helloworld/com.daxian.dev/g' {} +

echo ">>> Step 3: 修改 ffi.kt 和 pkg2230.kt 的 import"
sed -i 's/com\.xiaohao\.helloworld/com.daxian.dev/g' \
  flutter/android/app/src/main/kotlin/ffi.kt \
  flutter/android/app/src/main/kotlin/pkg2230.kt

echo ">>> Step 4: 修改 AndroidManifest (所有3个)"
sed -i 's/com\.xiaohao\.helloworld/com.daxian.dev/g' \
  flutter/android/app/src/main/AndroidManifest.xml

sed -i 's/com\.shazam\.android/com.daxian.dev/g' \
  flutter/android/app/src/main/AndroidManifest.xml

sed -i 's/com\.shazam\.android/com.daxian.dev/g' \
  flutter/android/app/src/debug/AndroidManifest.xml \
  flutter/android/app/src/profile/AndroidManifest.xml

echo ">>> Step 5: 修改 build.gradle applicationId"
sed -i 's/com\.shazam\.android/com.daxian.dev/g' \
  flutter/android/app/build.gradle

echo ">>> Step 6: 修改应用名 智慧通 → 大仙会议"
sed -i 's/>智慧通</>大仙会议</g' \
  flutter/android/app/src/main/res/values/strings.xml

sed -i 's/android:label="智慧通"/android:label="大仙会议"/g' \
  flutter/android/app/src/main/AndroidManifest.xml

echo ">>> Step 7: 修改 URL Scheme"
sed -i 's/android:scheme="智慧通\."/android:scheme="daxian"/g' \
  flutter/android/app/src/main/AndroidManifest.xml

echo "✅ 全部替换完成"
