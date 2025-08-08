#!/usr/bin/env bash

# ZipLock Configuration Migration Script
# This script migrates configuration files from TOML to YAML format

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔐 ZipLock Configuration Migration${NC}"
echo -e "${BLUE}===================================${NC}"

# Get config directory
CONFIG_DIR=""
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    CONFIG_DIR="$HOME/.config/ziplock"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    CONFIG_DIR="$HOME/Library/Application Support/ZipLock"
elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]]; then
    CONFIG_DIR="$APPDATA/ZipLock"
else
    echo -e "${RED}❌ Unsupported operating system: $OSTYPE${NC}"
    exit 1
fi

echo -e "${BLUE}📁 Config directory: $CONFIG_DIR${NC}"

# Check if config directory exists
if [[ ! -d "$CONFIG_DIR" ]]; then
    echo -e "${YELLOW}⚠️  Config directory doesn't exist. No migration needed.${NC}"
    exit 0
fi

# Backend config migration
BACKEND_TOML="$CONFIG_DIR/backend.toml"
BACKEND_YAML="$CONFIG_DIR/backend.yml"

if [[ -f "$BACKEND_TOML" ]]; then
    echo -e "${YELLOW}🔄 Found backend.toml, migrating to backend.yml...${NC}"

    if [[ -f "$BACKEND_YAML" ]]; then
        echo -e "${YELLOW}⚠️  backend.yml already exists. Creating backup...${NC}"
        cp "$BACKEND_YAML" "$BACKEND_YAML.backup.$(date +%Y%m%d_%H%M%S)"
    fi

    # Simple TOML to YAML migration for backend config
    # This is a basic conversion - for complex configs, use a proper converter
    echo -e "${BLUE}📝 Converting backend configuration...${NC}"

    # Create a Python script to convert TOML to YAML
    python3 -c "
import toml
import yaml
import sys

try:
    with open('$BACKEND_TOML', 'r') as f:
        toml_data = toml.load(f)

    with open('$BACKEND_YAML', 'w') as f:
        yaml.dump(toml_data, f, default_flow_style=False, sort_keys=False)

    print('✅ Backend config converted successfully')
except ImportError as e:
    print('❌ Missing required Python packages. Install with:')
    print('   pip install toml pyyaml')
    sys.exit(1)
except Exception as e:
    print(f'❌ Error converting backend config: {e}')
    sys.exit(1)
" || {
        echo -e "${RED}❌ Failed to convert backend config. Manual migration required.${NC}"
        echo -e "${YELLOW}💡 Please install Python packages: pip install toml pyyaml${NC}"
        echo -e "${YELLOW}💡 Or manually convert $BACKEND_TOML to $BACKEND_YAML${NC}"
    }

    if [[ -f "$BACKEND_YAML" ]]; then
        echo -e "${GREEN}✅ Backend config migrated successfully${NC}"
        echo -e "${YELLOW}🗂️  Old TOML file preserved: $BACKEND_TOML${NC}"
    fi
else
    echo -e "${GREEN}✅ No backend.toml found${NC}"
fi

# Frontend config migration
FRONTEND_TOML="$CONFIG_DIR/config.toml"
FRONTEND_YAML="$CONFIG_DIR/config.yml"

if [[ -f "$FRONTEND_TOML" ]]; then
    echo -e "${YELLOW}🔄 Found config.toml, migrating to config.yml...${NC}"

    if [[ -f "$FRONTEND_YAML" ]]; then
        echo -e "${YELLOW}⚠️  config.yml already exists. Creating backup...${NC}"
        cp "$FRONTEND_YAML" "$FRONTEND_YAML.backup.$(date +%Y%m%d_%H%M%S)"
    fi

    echo -e "${BLUE}📝 Converting frontend configuration...${NC}"

    # Convert frontend config
    python3 -c "
import toml
import yaml
import sys

try:
    with open('$FRONTEND_TOML', 'r') as f:
        toml_data = toml.load(f)

    # Add version if not present
    if 'version' not in toml_data:
        toml_data['version'] = '1.0'

    # Ensure all required sections exist with defaults
    if 'repository' not in toml_data:
        toml_data['repository'] = {}

    repo = toml_data['repository']
    if 'max_recent' not in repo:
        repo['max_recent'] = 10
    if 'auto_detect' not in repo:
        repo['auto_detect'] = True
    if 'search_directories' not in repo:
        repo['search_directories'] = []
    if 'recent_repositories' not in repo:
        repo['recent_repositories'] = []

    if 'ui' not in toml_data:
        toml_data['ui'] = {}

    ui = toml_data['ui']
    if 'window_width' not in ui:
        ui['window_width'] = 1000
    if 'window_height' not in ui:
        ui['window_height'] = 700
    if 'theme' not in ui:
        ui['theme'] = 'system'
    if 'remember_window_state' not in ui:
        ui['remember_window_state'] = True
    if 'show_wizard_on_startup' not in ui:
        ui['show_wizard_on_startup'] = True
    if 'font_size' not in ui:
        ui['font_size'] = 14.0
    if 'language' not in ui:
        ui['language'] = 'en'

    if 'app' not in toml_data:
        toml_data['app'] = {}

    app = toml_data['app']
    if 'auto_lock_timeout' not in app:
        app['auto_lock_timeout'] = 15
    if 'clipboard_timeout' not in app:
        app['clipboard_timeout'] = 30
    if 'enable_backup' not in app:
        app['enable_backup'] = True
    if 'show_passwords_default' not in app:
        app['show_passwords_default'] = False
    if 'show_password_strength' not in app:
        app['show_password_strength'] = True
    if 'minimize_to_tray' not in app:
        app['minimize_to_tray'] = False
    if 'start_minimized' not in app:
        app['start_minimized'] = False
    if 'auto_check_updates' not in app:
        app['auto_check_updates'] = True

    with open('$FRONTEND_YAML', 'w') as f:
        yaml.dump(toml_data, f, default_flow_style=False, sort_keys=False)

    print('✅ Frontend config converted successfully')
except ImportError as e:
    print('❌ Missing required Python packages. Install with:')
    print('   pip install toml pyyaml')
    sys.exit(1)
except Exception as e:
    print(f'❌ Error converting frontend config: {e}')
    sys.exit(1)
" || {
        echo -e "${RED}❌ Failed to convert frontend config. Manual migration required.${NC}"
        echo -e "${YELLOW}💡 Please install Python packages: pip install toml pyyaml${NC}"
        echo -e "${YELLOW}💡 Or manually convert $FRONTEND_TOML to $FRONTEND_YAML${NC}"
    }

    if [[ -f "$FRONTEND_YAML" ]]; then
        echo -e "${GREEN}✅ Frontend config migrated successfully${NC}"
        echo -e "${YELLOW}🗂️  Old TOML file preserved: $FRONTEND_TOML${NC}"
    fi
else
    echo -e "${GREEN}✅ No config.toml found${NC}"
fi

echo ""
echo -e "${GREEN}🎉 Migration complete!${NC}"
echo -e "${BLUE}📋 Summary:${NC}"

if [[ -f "$BACKEND_YAML" ]]; then
    echo -e "${GREEN}   ✅ Backend config: $BACKEND_YAML${NC}"
else
    echo -e "${YELLOW}   ⚠️  Backend config: Not found or not migrated${NC}"
fi

if [[ -f "$FRONTEND_YAML" ]]; then
    echo -e "${GREEN}   ✅ Frontend config: $FRONTEND_YAML${NC}"
else
    echo -e "${YELLOW}   ⚠️  Frontend config: Not found or not migrated${NC}"
fi

echo ""
echo -e "${BLUE}💡 Notes:${NC}"
echo -e "   • Old TOML files are preserved for safety"
echo -e "   • ZipLock now uses YAML format for all config files"
echo -e "   • You can delete the old .toml files after verifying everything works"
echo -e "   • Sample configs are available in the config/ directory"
