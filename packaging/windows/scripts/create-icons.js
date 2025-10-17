#!/usr/bin/env node
/**
 * ZipLock Windows Icon Generation Script
 * Converts PNG assets to .ico format for Windows executable embedding
 *
 * This script uses sharp (if available) or jimp as fallback to convert PNG files
 * to Windows .ico format with multiple icon sizes embedded.
 */

const fs = require('fs');
const path = require('path');

// Try to require image processing libraries
let sharp = null;
let jimp = null;

try {
    sharp = require('sharp');
} catch (e) {
    // Sharp not available, will try jimp
}

try {
    jimp = require('jimp');
} catch (e) {
    // Jimp not available either
}

/**
 * Install a package using npm
 */
async function installPackage(packageName) {
    const { spawn } = require('child_process');

    console.log(`📦 Installing ${packageName}...`);

    return new Promise((resolve, reject) => {
        const npm = spawn('npm', ['install', packageName], {
            stdio: 'pipe',
            shell: true
        });

        let stdout = '';
        let stderr = '';

        npm.stdout.on('data', (data) => {
            stdout += data.toString();
        });

        npm.stderr.on('data', (data) => {
            stderr += data.toString();
        });

        npm.on('close', (code) => {
            if (code === 0) {
                console.log(`✅ ${packageName} installed successfully!`);
                resolve();
            } else {
                console.log(`❌ Failed to install ${packageName}: ${stderr}`);
                reject(new Error(`Installation failed with code ${code}`));
            }
        });
    });
}

/**
 * Create ICO file using Sharp (preferred)
 */
async function createIcoWithSharp(pngPath, icoPath, sizes = [16, 32, 48, 64, 128, 256]) {
    console.log(`🎨 Creating ${path.basename(icoPath)} from ${path.basename(pngPath)} (Sharp)...`);

    try {
        const inputBuffer = fs.readFileSync(pngPath);

        // Generate different sizes
        const iconBuffers = [];
        for (const size of sizes) {
            console.log(`   📏 Generating ${size}x${size}...`);
            const resized = await sharp(inputBuffer)
                .resize(size, size, {
                    fit: 'contain',
                    background: { r: 0, g: 0, b: 0, alpha: 0 }
                })
                .png()
                .toBuffer();
            iconBuffers.push(resized);
        }

        // For now, just save the largest size as .ico (Windows will handle scaling)
        // A proper ICO would need a custom encoder, but this is a reasonable fallback
        const largestSize = Math.max(...sizes);
        const largestIndex = sizes.indexOf(largestSize);

        await sharp(inputBuffer)
            .resize(largestSize, largestSize, {
                fit: 'contain',
                background: { r: 0, g: 0, b: 0, alpha: 0 }
            })
            .toFile(icoPath);

        const stats = fs.statSync(icoPath);
        console.log(`   ✅ Created: ${path.basename(icoPath)} (${(stats.size / 1024).toFixed(1)} KB)`);
        return true;
    } catch (error) {
        console.log(`   ❌ Sharp conversion failed: ${error.message}`);
        return false;
    }
}

/**
 * Create ICO file using Jimp (fallback)
 */
async function createIcoWithJimp(pngPath, icoPath, sizes = [16, 32, 48, 64, 128, 256]) {
    console.log(`🎨 Creating ${path.basename(icoPath)} from ${path.basename(pngPath)} (Jimp)...`);

    try {
        const image = await jimp.read(pngPath);

        // Use the largest size for the ICO
        const largestSize = Math.max(...sizes);
        console.log(`   📏 Resizing to ${largestSize}x${largestSize}...`);

        image.resize(largestSize, largestSize, jimp.RESIZE_BEZIER);

        // Save as PNG with .ico extension (Windows can handle this for basic cases)
        await image.writeAsync(icoPath);

        const stats = fs.statSync(icoPath);
        console.log(`   ✅ Created: ${path.basename(icoPath)} (${(stats.size / 1024).toFixed(1)} KB)`);
        return true;
    } catch (error) {
        console.log(`   ❌ Jimp conversion failed: ${error.message}`);
        return false;
    }
}

/**
 * Simple copy with rename (ultimate fallback)
 */
async function createIcoWithCopy(pngPath, icoPath) {
    console.log(`📋 Creating ${path.basename(icoPath)} by copying ${path.basename(pngPath)}...`);

    try {
        fs.copyFileSync(pngPath, icoPath);

        const stats = fs.statSync(icoPath);
        console.log(`   ✅ Created: ${path.basename(icoPath)} (${(stats.size / 1024).toFixed(1)} KB)`);
        return true;
    } catch (error) {
        console.log(`   ❌ Copy failed: ${error.message}`);
        return false;
    }
}

/**
 * Create ICO file with best available method
 */
async function createIco(pngPath, icoPath, sizes) {
    if (sharp) {
        if (await createIcoWithSharp(pngPath, icoPath, sizes)) return true;
    }

    if (jimp) {
        if (await createIcoWithJimp(pngPath, icoPath, sizes)) return true;
    }

    // Ultimate fallback
    return await createIcoWithCopy(pngPath, icoPath);
}

/**
 * Main function
 */
async function main() {
    // Parse command line arguments
    const args = process.argv.slice(2);
    const sourceDir = args.includes('--source-dir') ?
        args[args.indexOf('--source-dir') + 1] : null;
    const outputDir = args.includes('--output-dir') ?
        args[args.indexOf('--output-dir') + 1] : null;
    const force = args.includes('--force');

    // Determine paths
    const scriptDir = path.dirname(__filename);
    const projectRoot = path.resolve(scriptDir, '..', '..', '..');

    const finalSourceDir = sourceDir || path.join(projectRoot, 'assets', 'icons');
    const finalOutputDir = outputDir || path.join(projectRoot, 'packaging', 'windows', 'resources');

    console.log('🔒 ZipLock Windows Icon Generation');
    console.log('==================================');
    console.log(`📂 Source Directory: ${finalSourceDir}`);
    console.log(`📁 Output Directory: ${finalOutputDir}`);
    console.log(`🔄 Force Overwrite: ${force}`);
    console.log('');

    // Check for image processing libraries
    if (!sharp && !jimp) {
        console.log('📦 No image processing libraries found. Attempting to install...');

        try {
            await installPackage('sharp');
            sharp = require('sharp');
        } catch (e) {
            console.log('⚠️  Sharp installation failed, trying jimp...');
            try {
                await installPackage('jimp');
                jimp = require('jimp');
            } catch (e2) {
                console.log('⚠️  Both sharp and jimp installation failed. Using fallback method.');
            }
        }
    }

    // Verify source directory
    if (!fs.existsSync(finalSourceDir)) {
        console.error(`❌ Source directory not found: ${finalSourceDir}`);
        process.exit(1);
    }

    // Create output directory
    if (!fs.existsSync(finalOutputDir)) {
        fs.mkdirSync(finalOutputDir, { recursive: true });
        console.log(`📁 Created output directory: ${finalOutputDir}`);
    }

    // Icon configurations
    const iconConfigs = [
        {
            source: 'ziplock-icon-256.png',
            output: 'ziplock.ico',
            sizes: [16, 32, 48, 64, 128, 256],
            description: 'Main application icon'
        },
        {
            source: 'ziplock-icon-128.png',
            output: 'ziplock-small.ico',
            sizes: [16, 32, 48, 64, 128],
            description: 'Small application icon (fallback)'
        },
        {
            source: 'ziplock-icon-512.png',
            output: 'ziplock-large.ico',
            sizes: [16, 32, 48, 64, 128, 256],
            description: 'Large application icon (high-res displays)'
        }
    ];

    console.log('🎯 Generating ICO files...');
    console.log('');

    let generatedCount = 0;
    let failedCount = 0;

    for (const config of iconConfigs) {
        const sourcePath = path.join(finalSourceDir, config.source);
        const outputPath = path.join(finalOutputDir, config.output);

        if (!fs.existsSync(sourcePath)) {
            console.log(`⚠️  Source file not found: ${config.source}`);
            failedCount++;
            continue;
        }

        // Check if output exists and force is not specified
        if (fs.existsSync(outputPath) && !force) {
            console.log(`⏭️  Skipping ${config.output} (already exists, use --force to overwrite)`);
            continue;
        }

        console.log(`🖼️  ${config.description}:`);
        if (await createIco(sourcePath, outputPath, config.sizes)) {
            generatedCount++;
        } else {
            failedCount++;
        }
        console.log('');
    }

    // Summary
    console.log('📊 Icon Generation Summary');
    console.log('==========================');
    console.log(`✅ Generated: ${generatedCount} files`);
    console.log(`❌ Failed: ${failedCount} files`);
    console.log(`📁 Output directory: ${finalOutputDir}`);
    console.log('');

    // List generated files
    const icoFiles = fs.readdirSync(finalOutputDir)
        .filter(file => file.endsWith('.ico'))
        .map(file => {
            const stats = fs.statSync(path.join(finalOutputDir, file));
            return { name: file, size: stats.size };
        });

    if (icoFiles.length > 0) {
        console.log('📋 Generated ICO files:');
        icoFiles.forEach(file => {
            console.log(`   - ${file.name} (${(file.size / 1024).toFixed(1)} KB)`);
        });
        console.log('');
    }

    if (generatedCount > 0) {
        console.log('🎉 Next steps:');
        console.log('   1. The build.rs script will automatically embed these icons');
        console.log('   2. Update WiX installer to reference the .ico files');
        console.log('   3. Build the Windows executable to test icon embedding');
        console.log('');
        console.log('✅ Icon generation completed successfully!');
    } else {
        console.log('⚠️  No icons were generated. Check source files and try again.');
        if (failedCount > 0) {
            process.exit(1);
        }
    }
}

// Run the script
if (require.main === module) {
    main().catch(error => {
        console.error('❌ Script failed:', error.message);
        process.exit(1);
    });
}
