# Advanced TypeScript Patterns

## Template Literal Types

### String Manipulation

```typescript
// Extract parts of strings
type ExtractRoute<T> = T extends `/${infer Route}` ? Route : never;
type Route = ExtractRoute<'/api/users'>; // 'api/users'

// Parse query parameters
type ParseQuery<T extends string> = T extends `${infer Key}=${infer Value}`
  ? { [K in Key]: Value }
  : never;

// CamelCase conversion
type CamelCase<S extends string> = S extends `${infer P1}_${infer P2}`
  ? `${Lowercase<P1>}${Capitalize<CamelCase<P2>>}`
  : Lowercase<S>;
```

### Type-Safe Route Building

```typescript
type RouteParams<T> = T extends `${string}:${infer Param}/${infer Rest}`
  ? { [K in Param | keyof RouteParams<`/${Rest}`>]: string }
  : T extends `${string}:${infer Param}`
  ? { [K in Param]: string }
  : {};

type Route = '/users/:userId/posts/:postId';
type Params = RouteParams<Route>; // { userId: string; postId: string }

function buildRoute<T extends string>(
  route: T,
  params: RouteParams<T>
): string {
  let result = route as string;
  for (const [key, value] of Object.entries(params)) {
    result = result.replace(`:${key}`, value);
  }
  return result;
}
```

## Recursive Types

### Deep Operations

```typescript
// Deep partial
type DeepPartial<T> = {
  [P in keyof T]?: T[P] extends object ? DeepPartial<T[P]> : T[P];
};

// Deep required
type DeepRequired<T> = {
  [P in keyof T]-?: T[P] extends object ? DeepRequired<T[P]> : T[P];
};

// Deep merge
type DeepMerge<T, U> = {
  [K in keyof T | keyof U]: K extends keyof U
    ? K extends keyof T
      ? T[K] extends object
        ? U[K] extends object
          ? DeepMerge<T[K], U[K]>
          : U[K]
        : U[K]
      : U[K]
    : K extends keyof T
    ? T[K]
    : never;
};
```

### Recursive Type Constraints

```typescript
// JSON value type
type JsonValue = string | number | boolean | null | JsonObject | JsonArray;
type JsonObject = { [key: string]: JsonValue };
type JsonArray = JsonValue[];

// Nested paths
type NestedKeyOf<ObjectType extends object> = {
  [Key in keyof ObjectType & (string | number)]: ObjectType[Key] extends object
    ? `${Key}` | `${Key}.${NestedKeyOf<ObjectType[Key]>}`
    : `${Key}`;
}[keyof ObjectType & (string | number)];
```

## Type-Level Computation

### Arithmetic at the Type Level

```typescript
// Length of tuple
type Length<T extends readonly unknown[]> = T['length'];

// Split string into tuple
type Split<S extends string, D extends string> = S extends `${infer H}${D}${infer T}`
  ? [H, ...Split<T, D>]
  : [S];

// Join tuple into string
type Join<T extends readonly string[], D extends string> = T extends readonly [
  infer F,
  ...infer R
]
  ? F extends string
    ? R extends readonly string[]
      ? R['length'] extends 0
        ? F
        : `${F}${D}${Join<R, D>}`
      : never
    : never
  : '';
```

### Type Guards and Narrowing

```typescript
// Type-safe narrowing
function isString(value: unknown): value is string {
  return typeof value === 'string';
}

function isNumber(value: unknown): value is number {
  return typeof value === 'number' && !isNaN(value);
}

function isArray<T>(value: unknown): value is T[] {
  return Array.isArray(value);
}

// Discriminated unions
type Success<T> = { status: 'success'; data: T };
type Failure<E> = { status: 'failure'; error: E };
type Result<T, E> = Success<T> | Failure<E>;

function handleResult<T, E>(result: Result<T, E>) {
  if (result.status === 'success') {
    // TypeScript knows result.data exists here
    console.log(result.data);
  } else {
    // TypeScript knows result.error exists here
    console.error(result.error);
  }
}
```

## Mapped Types

### Advanced Mapped Types

```typescript
// Make all properties writable
type Writable<T> = {
  -readonly [P in keyof T]: T[P];
};

// Make specific properties optional
type PartialBy<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>;

// Make specific properties required
type RequiredBy<T, K extends keyof T> = Omit<T, K> & Required<Pick<T, K>>;

// Function property types
type FunctionPropertyNames<T> = {
  [K in keyof T]: T[K] extends Function ? K : never;
}[keyof T];

// Non-function property types
type NonFunctionPropertyNames<T> = {
  [K in keyof T]: T[K] extends Function ? never : K;
}[keyof T];
```

## Const Assertions and Literal Types

### Preserving Literal Types

```typescript
// Const assertion
const config = {
  api: {
    baseUrl: 'https://api.example.com',
    timeout: 5000,
  },
} as const;

// Extract literal types
type ApiBaseUrl = typeof config.api.baseUrl; // 'https://api.example.com'
type ApiTimeout = typeof config.api.timeout; // 5000

// Named tuple with const assertion
const point = [10, 20] as const;
type Point = typeof point; // readonly [10, 20]
```

## Type Manipulation Utilities

### Advanced Utilities

```typescript
// Get all possible values from union
type UnionToIntersection<U> = (
  U extends any ? (x: U) => void : never
) extends (x: infer I) => void
  ? I
  : never;

// Extract function return type
type ReturnType<T> = T extends (...args: any[]) => infer R ? R : never;

// Extract function parameters
type Parameters<T> = T extends (...args: infer P) => any ? P : never;

// Extract constructor parameters
type ConstructorParameters<T> = T extends new (...args: infer P) => any ? P : never;

// Extract instance type
type InstanceType<T> = T extends new (...args: any[]) => infer R ? R : never;
```

## Practical Examples

### Type-Safe Event Emitter

```typescript
type EventMap = {
  user: { id: string; name: string };
  order: { orderId: string; total: number };
};

class TypedEventEmitter<Events extends Record<string, any>> {
  private listeners = new Map<keyof Events, Array<(data: any) => void>>();

  on<Event extends keyof Events>(
    event: Event,
    handler: (data: Events[Event]) => void
  ): void {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, []);
    }
    this.listeners.get(event)!.push(handler);
  }

  emit<Event extends keyof Events>(
    event: Event,
    data: Events[Event]
  ): void {
    const handlers = this.listeners.get(event);
    if (handlers) {
      handlers.forEach(handler => handler(data));
    }
  }
}

const emitter = new TypedEventEmitter<EventMap>();
emitter.on('user', (data) => {
  console.log(data.id, data.name); // Fully typed
});
```

### Type-Safe Builder Pattern

```typescript
class QueryBuilder<T extends Record<string, any>> {
  private filters: Partial<T> = {};

  where<K extends keyof T>(key: K, value: T[K]): this {
    this.filters[key] = value;
    return this;
  }

  build(): Partial<T> {
    return { ...this.filters };
  }
}

interface User {
  id: string;
  email: string;
  age: number;
}

const query = new QueryBuilder<User>()
  .where('email', 'user@example.com')
  .where('age', 25)
  .build();
```
