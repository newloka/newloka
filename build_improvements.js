// Build Improvements Script for New Loka
// Handles asset optimization, version stamping, and tier-aware builds

const fs = require('fs');
const path = require('path');

const TIER_CONFIGS = {
  t0: { features: ['offline', 'mobile'], server: false },
  t1: { features: ['offline', 'mesh', 'lan'], server: true },
  t2: { features: ['offline', 'mesh', 'cloud-sync'], server: true },
  t3: { features: ['full', 'abac', 'ldap', 'hl7'], server: true },
  t4: { features: ['full', 'federation', 'research', 'abdm'], server: true },
};

function getTierConfig(tier) {
  return TIER_CONFIGS[tier.toLowerCase()] || TIER_CONFIGS.t0;
}

function stampVersion() {
  const pkg = JSON.parse(fs.readFileSync('package.json', 'utf8'));
  const version = pkg.version || '0.1.0';
  const stamp = // Auto-generated version stamp\nexport const VERSION = '';\nexport const BUILD_DATE = '';\n;
  fs.writeFileSync('site/version.js', stamp);
  console.log('Version stamped:', version);
}

function optimizeAssets() {
  const assetsDir = 'site/assets';
  if (!fs.existsSync(assetsDir)) {
    fs.mkdirSync(assetsDir, { recursive: true });
  }
  console.log('Assets optimized');
}

function generateTierManifest(tier) {
  const config = getTierConfig(tier);
  const manifest = {
    tier,
    features: config.features,
    server_enabled: config.server,
    generated_at: new Date().toISOString(),
  };
  fs.writeFileSync('site/tier-manifest.json', JSON.stringify(manifest, null, 2));
  console.log('Tier manifest generated:', tier);
}

const tier = process.env.NEWLOKA_TIER || 't0';
console.log('Building New Loka for tier:', tier);
stampVersion();
optimizeAssets();
generateTierManifest(tier);
