---
name: typescript
description: Enforces modern TypeScript best practices for type-safe code, strict configuration, runtime validation, and clear type contracts. Use when writing or reviewing TypeScript/JavaScript code, configuring tsconfig.json, or when the user asks for TypeScript style guidance.
---

# TypeScript Best Practices

## Quick Start

Apply these rules by default when touching TypeScript/JavaScript:

1. Enable `strict: true` in `tsconfig.json` (most impactful change)
2. Define clear type contracts with interfaces/type aliases
3. Use `unknown` instead of `any`; narrow with type guards
4. Implement runtime validation for external data (APIs, user input)
5. Prefer string literal unions over numeric enums
6. Use named exports, avoid default exports

## Workflow (use this order)

1. Clarify scope: new code, refactor, or review.
2. Enforce strictness in `tsconfig.json` (or confirm it is already strict).
3. Define explicit type contracts for inputs/outputs (interfaces/type aliases).
4. Replace `any` with `unknown` + narrowing/guards.
5. Add runtime validation for external data boundaries.
6. Normalize exports/imports and tighten API surfaces.
7. Add or update tests for changed behavior.

## Review Checklist

- Strict mode enabled and no `any` leaks.
- Public types are explicit and documented.
- Runtime validation exists for untrusted inputs.
- Literal unions used instead of numeric enums.
- Named exports only; imports are organized.
- Tests cover edge cases and error paths.

## Local Resources

Use the always-applied rule file in this folder:
- `typescript.mdc` (authoritative TypeScript/JSX guidance; read when you need the full rule text)

## Configuration

**Essential `tsconfig.json` settings:**

```json
{
  "compilerOptions": {
    "strict": true,
    "noImplicitAny": true,
    "strictNullChecks": true,
    "strictPropertyInitialization": true,
    "allowJs": true,
    "checkJs": true
  }
}
```

## Type Definitions

### Interfaces for Object Shapes

Use interfaces for object shapes and class contracts. Define in `.d.ts` or `.ts` files, reference via JSDoc in `.js` files.

```typescript
interface UserProfile {
  id: string;
  name: string;
  email: string;
  age?: number;
}
```

### Type Aliases for Unions & Complex Types

Use type aliases for unions, intersections, and tuples.

```typescript
type ID = string | number;
type UserRole = "admin" | "user" | "guest";
type Coords = [number, number];
```

## Type Safety

### Avoid `any`, Use `unknown`

`any` bypasses all type checks. `unknown` forces narrowing before use.

❌ **Bad:**
```typescript
function processData(data: any) {
  console.log(data.foo.bar); // No type safety
}
```

✅ **Good:**
```typescript
function processData(data: unknown) {
  if (typeof data === 'object' && data !== null && 'foo' in data) {
    const typed = data as { foo: { bar: string } };
    console.log(typed.foo.bar);
  }
}
```

## Runtime Validation

TypeScript's checks are compile-time only. For external data (APIs, user input), implement runtime validation with type guards.

### Type Guards for Primitives

```typescript
function isString(value: unknown): value is string {
  return typeof value === 'string';
}

function isNumber(value: unknown): value is number {
  return typeof value === 'number' && !isNaN(value);
}
```

### Type Guards for Complex Objects

```typescript
interface Product {
  id: string;
  name: string;
  price: number;
}

function isProduct(obj: unknown): obj is Product {
  return (
    typeof obj === 'object' && obj !== null &&
    'id' in obj && typeof obj.id === 'string' &&
    'name' in obj && typeof obj.name === 'string' &&
    'price' in obj && typeof obj.price === 'number'
  );
}
```

## Enums and Unions

Prefer string literal unions over numeric enums for better type safety and simpler runtime representation.

❌ **Bad:**
```typescript
enum UserStatus {
  Active,    // 0
  Inactive,  // 1
  Pending    // 2
}
```

✅ **Good:**
```typescript
type UserStatus = 'active' | 'inactive' | 'pending';

// If enum is necessary, use const string enum:
const enum UserRole {
  Admin = "admin",
  User = "user",
  Guest = "guest"
}
```

## Generics

Use generics for reusable, type-safe functions and components.

```typescript
function identity<T>(arg: T): T {
  return arg;
}

const num = identity(123);  // number
const str = identity("hello"); // string
```

## Code Organization

### Named Exports (No Default Exports)

Named exports promote explicit imports and easier refactoring.

❌ **Bad:**
```typescript
export default class UserService { }
```

✅ **Good:**
```typescript
export class UserService { }
export const DEFAULT_USER = { };
```

### Organized Imports

Group imports: libraries → absolute paths → relative paths. Sort alphabetically.

```typescript
import React from 'react';
import { useSelector } from 'react-redux';

import { API_URL } from 'config/constants';
import { selectUser } from 'store/selectors';

import { Button } from './components/Button';
import { formatCurrency } from '../utils/formatters';
```

## Examples

**Example 1: Type guard for API response**

Input: "Validate and process API response"
Output:
```typescript
interface ApiUser {
  id: string;
  name: string;
  email: string;
}

function isApiUser(obj: unknown): obj is ApiUser {
  return (
    typeof obj === 'object' && obj !== null &&
    'id' in obj && typeof obj.id === 'string' &&
    'name' in obj && typeof obj.name === 'string' &&
    'email' in obj && typeof obj.email === 'string'
  );
}

async function fetchUser(id: string): Promise<ApiUser | null> {
  const response = await fetch(`/api/users/${id}`);
  const data: unknown = await response.json();
  
  if (isApiUser(data)) {
    return data;
  }
  return null;
}
```

**Example 2: String literal union type**

Input: "Status type with specific values"
Output:
```typescript
type OrderStatus = 'pending' | 'processing' | 'shipped' | 'delivered';

function updateOrderStatus(orderId: string, status: OrderStatus) {
  // TypeScript ensures only valid status values
  console.log(`Order ${orderId} -> ${status}`);
}

updateOrderStatus('123', 'shipped'); // ✅ OK
// updateOrderStatus('123', 'invalid'); // ❌ Type error
```

**Example 3: Generic utility function**

Input: "Reusable function that works with any array type"
Output:
```typescript
function getFirst<T>(items: T[]): T | undefined {
  return items[0];
}

const firstNum = getFirst([1, 2, 3]); // number | undefined
const firstStr = getFirst(['a', 'b']); // string | undefined
```

**Example 4: JSDoc types for JavaScript files**

Input: "Type-check JavaScript file with JSDoc"
Output:
```javascript
// types.d.ts
export interface UserProfile {
  id: string;
  name: string;
  email: string;
}

// user-service.js
/**
 * @typedef {import('./types').UserProfile} UserProfile
 */

/**
 * @param {UserProfile} user
 * @returns {string}
 */
export function greetUser(user) {
  return `Hello, ${user.name}!`;
}
```
