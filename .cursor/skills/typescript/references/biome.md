# Biome Configuration Guide

## Overview

Biome is a fast, all-in-one toolchain for web projects that replaces ESLint and Prettier. It's written in Rust, making it significantly faster than JavaScript-based alternatives.

## Installation

```bash
# npm
npm install --save-dev @biomejs/biome

# pnpm
pnpm add -D @biomejs/biome

# yarn
yarn add --dev @biomejs/biome

# Initialize configuration
npx @biomejs/biome init
```

## Configuration File Structure

### Basic Configuration (`biome.json`)

```json
{
  "$schema": "https://biomejs.dev/schemas/1.9.4/schema.json",
  "organizeImports": {
    "enabled": true
  },
  "linter": {
    "enabled": true
  },
  "formatter": {
    "enabled": true
  }
}
```

### Complete Production Configuration

```json
{
  "$schema": "https://biomejs.dev/schemas/1.9.4/schema.json",
  "vcs": {
    "enabled": true,
    "clientKind": "git",
    "useIgnoreFile": true
  },
  "files": {
    "ignore": [
      "node_modules",
      "dist",
      "build",
      ".next",
      ".turbo",
      "coverage",
      "*.min.js",
      "*.bundle.js"
    ],
    "include": ["src/**/*", "*.ts", "*.tsx", "*.js", "*.jsx"]
  },
  "organizeImports": {
    "enabled": true
  },
  "linter": {
    "enabled": true,
    "rules": {
      "recommended": true,
      "correctness": {
        "noUnusedVariables": "error",
        "useExhaustiveDependencies": "warn",
        "noUnusedImports": "error"
      },
      "style": {
        "useImportType": "error",
        "useConst": "error",
        "noParameterAssign": "error",
        "useNamingConvention": {
          "level": "error",
          "options": {
            "strictCase": false,
            "conventions": [
              {
                "selector": {
                  "kind": "variable",
                  "const": true,
                  "global": true
                },
                "formats": ["CONSTANT_CASE"]
              },
              {
                "selector": {
                  "kind": "typeLike"
                },
                "formats": ["PascalCase"]
              }
            ]
          }
        }
      },
      "suspicious": {
        "noExplicitAny": "error",
        "noArrayIndexKey": "warn",
        "noAssignInExpressions": "error",
        "noDoubleEquals": "error",
        "noNegationElse": "off"
      },
      "performance": {
        "noDelete": "error"
      },
      "complexity": {
        "noBannedTypes": "warn",
        "noForEach": "off",
        "useSimplifiedLogicExpression": "warn",
        "useOptionalChain": "error"
      },
      "security": {
        "noDangerouslySetInnerHtml": "error"
      }
    }
  },
  "formatter": {
    "enabled": true,
    "formatWithErrors": false,
    "indentStyle": "space",
    "indentWidth": 2,
    "lineWidth": 100,
    "lineEnding": "lf"
  },
  "javascript": {
    "formatter": {
      "quoteStyle": "single",
      "jsxQuoteStyle": "double",
      "trailingCommas": "es5",
      "semicolons": "always",
      "arrowParentheses": "always",
      "bracketSpacing": true,
      "bracketSameLine": false
    }
  },
  "overrides": [
    {
      "include": ["*.test.ts", "*.test.tsx", "*.spec.ts", "*.spec.tsx"],
      "linter": {
        "rules": {
          "suspicious": {
            "noExplicitAny": "off"
          },
          "complexity": {
            "noBannedTypes": "off"
          }
        }
      }
    },
    {
      "include": ["*.config.ts", "*.config.js"],
      "linter": {
        "rules": {
          "suspicious": {
            "noExplicitAny": "off"
          }
        }
      }
    }
  ]
}
```

## Rule Categories

### Correctness Rules

Prevent bugs and maintain code correctness:

```json
{
  "correctness": {
    "noUnusedVariables": "error",
    "noUnusedImports": "error",
    "useExhaustiveDependencies": "warn",
    "useHookAtTopLevel": "error",
    "noUnusedFunctionParameters": "warn"
  }
}
```

### Style Rules

Enforce consistent code style:

```json
{
  "style": {
    "useImportType": "error",
    "useConst": "error",
    "useShorthandArrayType": "error",
    "useNamingConvention": "error",
    "noNegationElse": "off"
  }
}
```

### Suspicious Rules

Catch potentially buggy patterns:

```json
{
  "suspicious": {
    "noExplicitAny": "error",
    "noArrayIndexKey": "warn",
    "noAssignInExpressions": "error",
    "noDoubleEquals": "error",
    "noShadowRestrictedNames": "error"
  }
}
```

### Performance Rules

Optimize for runtime performance:

```json
{
  "performance": {
    "noDelete": "error",
    "noAccumulatingSpread": "warn"
  }
}
```

### Complexity Rules

Keep code simple and maintainable:

```json
{
  "complexity": {
    "noForEach": "off",
    "useSimplifiedLogicExpression": "warn",
    "useOptionalChain": "error",
    "noBannedTypes": "warn"
  }
}
```

### Security Rules

Prevent security vulnerabilities:

```json
{
  "security": {
    "noDangerouslySetInnerHtml": "error",
    "noGlobalEval": "error"
  }
}
```

## TypeScript-Specific Configuration

### Type-Aware Rules

```json
{
  "javascript": {
    "parser": {
      "unsafeParameterDecoratorsEnabled": false
    },
    "formatter": {
      "quoteStyle": "single",
      "jsxQuoteStyle": "double"
    }
  },
  "overrides": [
    {
      "include": ["*.ts", "*.tsx"],
      "linter": {
        "rules": {
          "style": {
            "useImportType": "error"
          }
        }
      }
    }
  ]
}
```

## Import Organization

Biome automatically organizes imports according to your configuration:

```json
{
  "organizeImports": {
    "enabled": true
  }
}
```

**Import organization order:**
1. External packages (node_modules)
2. Internal packages (workspace packages)
3. Absolute imports (from tsconfig paths)
4. Relative imports

**Example transformation:**
```typescript
// Before
import { Component } from './component';
import React from 'react';
import { useState } from 'react';
import { utils } from '@/utils';

// After (organized)
import React, { useState } from 'react';

import { utils } from '@/utils';

import { Component } from './component';
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Lint and Format Check

on:
  pull_request:
    branches: [main]
  push:
    branches: [main]

jobs:
  biome:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
      - run: npm ci
      - uses: biomejs/setup-biome@v2
        with:
          version: latest
      - run: biome check .
```

### Pre-commit Hook (Husky)

```bash
# .husky/pre-commit
#!/usr/bin/env sh
. "$(dirname -- "$0")/_/husky.sh"

npx biome check --write .
```

## VS Code Integration

### Settings

```json
{
  "[typescript]": {
    "editor.defaultFormatter": "biomejs.biome",
    "editor.formatOnSave": true,
    "editor.codeActionsOnSave": {
      "quickfix.biome": "explicit",
      "source.organizeImports.biome": "explicit"
    }
  },
  "[typescriptreact]": {
    "editor.defaultFormatter": "biomejs.biome",
    "editor.formatOnSave": true,
    "editor.codeActionsOnSave": {
      "quickfix.biome": "explicit",
      "source.organizeImports.biome": "explicit"
    }
  },
  "[javascript]": {
    "editor.defaultFormatter": "biomejs.biome",
    "editor.formatOnSave": true
  },
  "[javascriptreact]": {
    "editor.defaultFormatter": "biomejs.biome",
    "editor.formatOnSave": true
  },
  "[json]": {
    "editor.defaultFormatter": "biomejs.biome"
  },
  "[jsonc]": {
    "editor.defaultFormatter": "biomejs.biome"
  }
}
```

## Migration from ESLint + Prettier

### Step 1: Install Biome

```bash
npm install --save-dev @biomejs/biome
```

### Step 2: Create Configuration

Use the migration guide or start with recommended settings:

```bash
npx @biomejs/biome init
```

### Step 3: Replace Scripts

Update `package.json`:

```json
{
  "scripts": {
    "lint": "biome check .",
    "lint:fix": "biome check --write .",
    "format": "biome format --write ."
  }
}
```

### Step 4: Remove Old Tools

```bash
npm uninstall eslint prettier @typescript-eslint/eslint-plugin @typescript-eslint/parser eslint-config-prettier eslint-plugin-prettier
```

### Step 5: Update CI/CD

Replace ESLint/Prettier checks with Biome checks.

## Common Patterns

### Ignoring Files

```json
{
  "files": {
    "ignore": [
      "node_modules",
      "dist",
      "build",
      "coverage",
      "*.generated.ts"
    ]
  }
}
```

### File-Specific Rules

```json
{
  "overrides": [
    {
      "include": ["*.test.ts"],
      "linter": {
        "rules": {
          "suspicious": {
            "noExplicitAny": "off"
          }
        }
      }
    },
    {
      "include": ["scripts/**/*"],
      "formatter": {
        "enabled": false
      }
    }
  ]
}
```

### Workspace Configuration

For monorepos, configure at root and extend in packages:

```json
// biome.json (root)
{
  "$schema": "https://biomejs.dev/schemas/1.9.4/schema.json",
  "files": {
    "ignore": ["node_modules", "dist"]
  }
}

// packages/my-package/biome.json
{
  "$extends": ["../../biome.json"],
  "files": {
    "ignore": ["node_modules", "dist", ".next"]
  }
}
```

## Performance Tips

1. **Use `.biomeignore`** for large directories:
   ```
   node_modules/
   dist/
   build/
   .next/
   coverage/
   ```

2. **Enable VCS integration** to respect `.gitignore`:
   ```json
   {
     "vcs": {
       "enabled": true,
       "useIgnoreFile": true
     }
   }
   ```

3. **Run in parallel** for large codebases:
   ```bash
   # Check multiple directories
   biome check src/ tests/ scripts/
   ```

## Best Practices

1. **Start with recommended rules** and customize as needed
2. **Use overrides** for file-specific configurations
3. **Enable format on save** in your editor
4. **Run in CI/CD** to enforce consistency
5. **Update configuration incrementally** rather than all at once
6. **Document rule changes** in your team's style guide

## Troubleshooting

### Conflicts with TypeScript

If TypeScript and Biome disagree, Biome takes precedence for formatting. For type errors, TypeScript is the source of truth.

### Migration Issues

When migrating from ESLint:
1. Start with Biome's recommended rules
2. Gradually add stricter rules
3. Use `--diagnostic-level=info` to see what rules are applied

### Performance

If Biome is slow:
- Check that large directories are in `.biomeignore`
- Ensure VCS integration is enabled
- Use file-specific ignores in `biome.json`
