#!/usr/bin/env node
/**
 * 从 icon.jpg 生成 Tauri 所需的全套图标
 * 包括：桌面图标、托盘图标、Windows/macOS/Linux/Android/iOS 各平台图标
 *
 * 使用方法: node scripts/generate-icons.js
 */

const sharp = require('sharp');
const path = require('path');
const fs = require('fs');

const ICONS_DIR = path.join(__dirname, '../src-tauri/icons');
const SOURCE_ICON = path.join(ICONS_DIR, 'icon.jpg');

async function generateIcons() {
  console.log('🎨 开始生成图标...');
  console.log(`📁 源文件: ${SOURCE_ICON}`);

  if (!fs.existsSync(SOURCE_ICON)) {
    console.error('❌ 源图标文件不存在:', SOURCE_ICON);
    process.exit(1);
  }

  const sourceImage = sharp(SOURCE_ICON);

  // 通用 PNG 图标
  const pngSizes = [32, 64, 128, 256, 512];
  for (const size of pngSizes) {
    await sourceImage.clone().resize(size, size).png().toFile(path.join(ICONS_DIR, `${size}x${size}.png`));
    console.log(`✅ ${size}x${size}.png`);
  }

  // macOS @2x 图标
  await sourceImage.clone().resize(256, 256).png().toFile(path.join(ICONS_DIR, '128x128@2x.png'));
  console.log('✅ 128x128@2x.png');

  // 主 icon.png
  await sourceImage.clone().resize(512, 512).png().toFile(path.join(ICONS_DIR, 'icon.png'));
  console.log('✅ icon.png');

  // Windows Store Logo 图标
  const windowsSizes = [
    { name: 'Square30x30Logo', size: 30 },
    { name: 'Square44x44Logo', size: 44 },
    { name: 'Square71x71Logo', size: 71 },
    { name: 'Square89x89Logo', size: 89 },
    { name: 'Square107x107Logo', size: 107 },
    { name: 'Square142x142Logo', size: 142 },
    { name: 'Square150x150Logo', size: 150 },
    { name: 'Square284x284Logo', size: 284 },
    { name: 'Square310x310Logo', size: 310 },
    { name: 'StoreLogo', size: 50 },
  ];
  for (const { name, size } of windowsSizes) {
    await sourceImage.clone().resize(size, size).png().toFile(path.join(ICONS_DIR, `${name}.png`));
    console.log(`✅ ${name}.png`);
  }

  // ICO 文件 (Windows)
  try {
    const { default: pngToIco } = require('png-to-ico');
    const pngFiles = ['256x256.png', '128x128.png', '64x64.png', '32x32.png'].map(f => path.join(ICONS_DIR, f));
    const icoBuffer = await pngToIco(pngFiles);
    fs.writeFileSync(path.join(ICONS_DIR, 'icon.ico'), icoBuffer);
    console.log('✅ icon.ico');
  } catch (e) {
    console.log('⚠️  跳过 icon.ico (需要安装 png-to-ico)');
  }

  // macOS 托盘图标
  const trayDir = path.join(ICONS_DIR, 'tray/macos');
  if (fs.existsSync(trayDir)) {
    await sourceImage.clone().resize(22, 22).png().toFile(path.join(trayDir, 'statusTemplate.png'));
    await sourceImage.clone().resize(44, 44).png().toFile(path.join(trayDir, 'statusTemplate@2x.png'));
    console.log('✅ macOS 托盘图标');
  }

  // Android 图标
  const androidSizes = [
    { dir: 'mipmap-mdpi', size: 48 },
    { dir: 'mipmap-hdpi', size: 72 },
    { dir: 'mipmap-xhdpi', size: 96 },
    { dir: 'mipmap-xxhdpi', size: 144 },
    { dir: 'mipmap-xxxhdpi', size: 192 },
  ];
  for (const { dir, size } of androidSizes) {
    const androidDir = path.join(ICONS_DIR, 'android', dir);
    if (fs.existsSync(androidDir)) {
      await sourceImage.clone().resize(size, size).png().toFile(path.join(androidDir, 'ic_launcher.png'));
      await sourceImage.clone().resize(size, size).png().toFile(path.join(androidDir, 'ic_launcher_round.png'));
      await sourceImage.clone().resize(size, size).png().toFile(path.join(androidDir, 'ic_launcher_foreground.png'));
    }
  }
  console.log('✅ Android 图标');

  // iOS 图标
  const iosDir = path.join(ICONS_DIR, 'ios');
  if (fs.existsSync(iosDir)) {
    const iosSizes = [
      { name: 'AppIcon-20x20@1x', size: 20 },
      { name: 'AppIcon-20x20@2x', size: 40 },
      { name: 'AppIcon-20x20@2x-1', size: 40 },
      { name: 'AppIcon-20x20@3x', size: 60 },
      { name: 'AppIcon-29x29@1x', size: 29 },
      { name: 'AppIcon-29x29@2x', size: 58 },
      { name: 'AppIcon-29x29@2x-1', size: 58 },
      { name: 'AppIcon-29x29@3x', size: 87 },
      { name: 'AppIcon-40x40@1x', size: 40 },
      { name: 'AppIcon-40x40@2x', size: 80 },
      { name: 'AppIcon-40x40@2x-1', size: 80 },
      { name: 'AppIcon-40x40@3x', size: 120 },
      { name: 'AppIcon-60x60@2x', size: 120 },
      { name: 'AppIcon-60x60@3x', size: 180 },
      { name: 'AppIcon-76x76@1x', size: 76 },
      { name: 'AppIcon-76x76@2x', size: 152 },
      { name: 'AppIcon-83.5x83.5@2x', size: 167 },
      { name: 'AppIcon-512@2x', size: 1024 },
    ];
    for (const { name, size } of iosSizes) {
      await sourceImage.clone().resize(size, size).png().toFile(path.join(iosDir, `${name}.png`));
    }
    console.log('✅ iOS 图标');
  }

  console.log('\n🎉 全部图标生成完成！');
  console.log('⚠️  注意: icon.icns 需要在 macOS 上使用 iconutil 命令生成');
}

generateIcons().catch(console.error);
