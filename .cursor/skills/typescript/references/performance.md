# TypeScript Performance Optimization

## Bundle Size Optimization

### Tree Shaking

```typescript
// ✅ Good - Named exports, tree-shakeable
export function utility1() {}
export function utility2() {}

// ❌ Bad - Default export with side effects
export default {
  utility1() {},
  utility2() {},
};

// Import only what you need
import { utility1 } from './utils'; // Only utility1 in bundle
```

### Code Splitting

```typescript
// Dynamic imports for code splitting
const loadHeavyModule = async () => {
  const { HeavyModule } = await import('./heavy-module');
  return HeavyModule;
};

// Lazy route loading
const routes = {
  '/dashboard': () => import('./pages/Dashboard'),
  '/settings': () => import('./pages/Settings'),
};

// Use React.lazy for components
const LazyComponent = React.lazy(() => import('./HeavyComponent'));
```

### Bundle Analysis

```typescript
// Analyze bundle size
import { BundleAnalyzerPlugin } from 'webpack-bundle-analyzer';

// webpack.config.js
module.exports = {
  plugins: [
    new BundleAnalyzerPlugin({
      analyzerMode: 'static',
      reportFilename: 'bundle-report.html',
    }),
  ],
};
```

## Runtime Performance

### Avoid Type Guards in Hot Paths

```typescript
// ❌ Bad - Type guard in hot path
function processItems(items: unknown[]) {
  return items.filter(item => isItem(item));
}

// ✅ Good - Pre-validate once
function processItems(items: Item[]) {
  return items; // Already validated
}
```

### Memoization

```typescript
// Simple memoization
function memoize<Args extends unknown[], Return>(
  fn: (...args: Args) => Return
): (...args: Args) => Return {
  const cache = new Map<string, Return>();
  return (...args: Args) => {
    const key = JSON.stringify(args);
    if (!cache.has(key)) {
      cache.set(key, fn(...args));
    }
    return cache.get(key)!;
  };
}

// WeakMap for object keys (better memory management)
function memoizeWeak<Args extends object[], Return>(
  fn: (...args: Args) => Return
): (...args: Args) => Return {
  const cache = new WeakMap<Args[0], Return>();
  return (...args: Args) => {
    if (!cache.has(args[0])) {
      cache.set(args[0], fn(...args));
    }
    return cache.get(args[0])!;
  };
}
```

### Batch Operations

```typescript
// Batch database operations
async function batchInsert<T>(
  items: T[],
  batchSize: number,
  inserter: (batch: T[]) => Promise<void>
): Promise<void> {
  for (let i = 0; i < items.length; i += batchSize) {
    await inserter(items.slice(i, i + batchSize));
  }
}

// Debounce/throttle expensive operations
function debounce<T extends (...args: any[]) => any>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout>;
  return (...args: Parameters<T>) => {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn(...args), delay);
  };
}

function throttle<T extends (...args: any[]) => any>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let lastCall = 0;
  return (...args: Parameters<T>) => {
    const now = Date.now();
    if (now - lastCall >= delay) {
      lastCall = now;
      fn(...args);
    }
  };
}
```

### Object Pooling

```typescript
// Reuse objects to reduce GC pressure
class ObjectPool<T> {
  private pool: T[] = [];
  
  constructor(
    private readonly factory: () => T,
    private readonly reset: (obj: T) => void
  ) {}
  
  acquire(): T {
    return this.pool.pop() ?? this.factory();
  }
  
  release(obj: T): void {
    this.reset(obj);
    this.pool.push(obj);
  }
}

// Usage for temporary objects
const pool = new ObjectPool<Buffer>(
  () => Buffer.alloc(1024),
  (buf) => buf.fill(0)
);
```

## Compile-Time Optimizations

### Const Enums

```typescript
// ✅ Good - Inlined at compile time
const enum Status {
  Active = 1,
  Inactive = 2,
}

// ❌ Bad - Runtime lookup
enum Status {
  Active = 1,
  Inactive = 2,
}
```

### Type-Only Imports

```typescript
// Type-only imports are stripped from output
import type { User } from './types';
import { createUser } from './factory';

// vs runtime import
import { User, createUser } from './module'; // User type remains in output
```

### Project References

```typescript
// tsconfig.json - Split into multiple projects
{
  "compilerOptions": {
    "composite": true,
    "incremental": true
  },
  "references": [
    { "path": "./packages/core" },
    { "path": "./packages/utils" }
  ]
}
```

## Memory Management

### Weak References

```typescript
// Use WeakMap/WeakSet for metadata that shouldn't prevent GC
const metadata = new WeakMap<object, Metadata>();

function attachMetadata(obj: object, data: Metadata): void {
  metadata.set(obj, data);
}

// Object can be GC'd, metadata will be too
```

### Avoid Memory Leaks

```typescript
// Clean up event listeners
class EventEmitter {
  private listeners = new Map<string, Set<Function>>();
  
  on(event: string, handler: Function): () => void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }
    this.listeners.get(event)!.add(handler);
    
    // Return cleanup function
    return () => {
      this.listeners.get(event)?.delete(handler);
    };
  }
}

// Usage
const cleanup = emitter.on('event', handler);
// Later: cleanup(); // Remove listener
```

## Profiling and Measurement

### Performance Measurement

```typescript
// Measure execution time
function measureTime<T>(label: string, fn: () => T): T {
  const start = performance.now();
  const result = fn();
  const end = performance.now();
  console.log(`${label}: ${end - start}ms`);
  return result;
}

// Async version
async function measureTimeAsync<T>(
  label: string,
  fn: () => Promise<T>
): Promise<T> {
  const start = performance.now();
  const result = await fn();
  const end = performance.now();
  console.log(`${label}: ${end - start}ms`);
  return result;
}

// Usage
const result = measureTime('expensive operation', () => {
  // ... expensive code
});
```

### Memory Profiling

```typescript
// Check memory usage (Node.js)
function logMemoryUsage(): void {
  const usage = process.memoryUsage();
  console.log({
    rss: `${Math.round(usage.rss / 1024 / 1024)}MB`,
    heapTotal: `${Math.round(usage.heapTotal / 1024 / 1024)}MB`,
    heapUsed: `${Math.round(usage.heapUsed / 1024 / 1024)}MB`,
  });
}
```

## Best Practices Summary

1. **Bundle Size**:
   - Use named exports for tree-shaking
   - Code split heavy modules
   - Analyze bundle regularly

2. **Runtime**:
   - Memoize expensive computations
   - Batch operations
   - Avoid type guards in hot paths

3. **Compile-Time**:
   - Use const enums for constants
   - Import types separately
   - Use project references for large codebases

4. **Memory**:
   - Clean up resources (event listeners, timers)
   - Use WeakMap/WeakSet for metadata
   - Profile memory usage regularly
