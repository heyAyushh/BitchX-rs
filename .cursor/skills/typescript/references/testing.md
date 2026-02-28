# TypeScript Testing Strategies

## Test Organization

### Structure

```
src/
├── features/
│   ├── users/
│   │   ├── user.service.ts
│   │   └── user.service.test.ts
│   └── orders/
└── __tests__/
    ├── integration/
    └── e2e/
```

### Test Categories

```typescript
// Unit tests - Test single units in isolation
describe('UserService', () => {
  it('should create user', async () => {
    // Test single method
  });
});

// Integration tests - Test component interactions
describe('UserService + Repository', () => {
  it('should persist user to database', async () => {
    // Test multiple components working together
  });
});

// E2E tests - Test full user flows
describe('User Registration Flow', () => {
  it('should register new user via API', async () => {
    // Test complete flow
  });
});
```

## Mocking Strategies

### Dependency Injection for Testing

```typescript
// Production code - inject dependencies
class UserService {
  constructor(
    private readonly userRepo: UserRepository,
    private readonly emailService: EmailService
  ) {}
  
  async registerUser(email: string): Promise<User> {
    const user = await this.userRepo.save({ email });
    await this.emailService.send(email, 'Welcome', '...');
    return user;
  }
}

// Test - inject mocks
describe('UserService', () => {
  let mockRepo: jest.Mocked<UserRepository>;
  let mockEmailService: jest.Mocked<EmailService>;
  let userService: UserService;
  
  beforeEach(() => {
    mockRepo = {
      save: jest.fn(),
      findById: jest.fn(),
    } as any;
    
    mockEmailService = {
      send: jest.fn(),
    } as any;
    
    userService = new UserService(mockRepo, mockEmailService);
  });
  
  it('should register user', async () => {
    const mockUser = { id: '1', email: 'test@example.com' };
    mockRepo.save.mockResolvedValue(mockUser);
    
    const result = await userService.registerUser('test@example.com');
    
    expect(result).toEqual(mockUser);
    expect(mockRepo.save).toHaveBeenCalledWith({ email: 'test@example.com' });
    expect(mockEmailService.send).toHaveBeenCalled();
  });
});
```

### Manual Mocks

```typescript
// Create mock implementations
class MockUserRepository implements UserRepository {
  private users = new Map<string, User>();
  
  async findById(id: string): Promise<User | null> {
    return this.users.get(id) ?? null;
  }
  
  async save(user: User): Promise<User> {
    this.users.set(user.id, user);
    return user;
  }
  
  // Helper for tests
  addUser(user: User): void {
    this.users.set(user.id, user);
  }
}

// Use in tests
describe('UserService', () => {
  let mockRepo: MockUserRepository;
  let userService: UserService;
  
  beforeEach(() => {
    mockRepo = new MockUserRepository();
    userService = new UserService(mockRepo);
  });
  
  it('should find existing user', async () => {
    const user = { id: '1', email: 'test@example.com' };
    mockRepo.addUser(user);
    
    const result = await userService.getUser('1');
    
    expect(result).toEqual(user);
  });
});
```

### Partial Mocks

```typescript
// Mock only specific methods
const realRepository = new PostgresUserRepository();
const mockRepository = {
  ...realRepository,
  findById: jest.fn(),
} as unknown as UserRepository;

// Use real methods for some, mocked for others
mockRepository.save = realRepository.save.bind(realRepository);
```

## Test Utilities

### Factories

```typescript
// Create test data easily
class UserFactory {
  static create(overrides?: Partial<User>): User {
    return {
      id: '1',
      email: 'test@example.com',
      name: 'Test User',
      ...overrides,
    };
  }
  
  static createMany(count: number): User[] {
    return Array.from({ length: count }, (_, i) =>
      this.create({ id: String(i + 1) })
    );
  }
}

// Usage
const user = UserFactory.create({ email: 'custom@example.com' });
const users = UserFactory.createMany(10);
```

### Builders

```typescript
// Fluent API for building test objects
class UserBuilder {
  private user: Partial<User> = {};
  
  withId(id: string): this {
    this.user.id = id;
    return this;
  }
  
  withEmail(email: string): this {
    this.user.email = email;
    return this;
  }
  
  withName(name: string): this {
    this.user.name = name;
    return this;
  }
  
  build(): User {
    return {
      id: this.user.id ?? '1',
      email: this.user.email ?? 'test@example.com',
      name: this.user.name ?? 'Test User',
    };
  }
}

// Usage
const user = new UserBuilder()
  .withEmail('custom@example.com')
  .withName('Custom Name')
  .build();
```

### Test Helpers

```typescript
// Async test helpers
async function waitFor(
  condition: () => boolean,
  timeout = 1000
): Promise<void> {
  const start = Date.now();
  while (!condition()) {
    if (Date.now() - start > timeout) {
      throw new Error('Condition not met within timeout');
    }
    await new Promise(resolve => setTimeout(resolve, 10));
  }
}

// Assertion helpers
function assertUser(user: unknown): asserts user is User {
  if (!user || typeof user !== 'object') {
    throw new Error('Not a user');
  }
  if (!('id' in user) || !('email' in user)) {
    throw new Error('Invalid user structure');
  }
}

// Usage in tests
test('user has required fields', () => {
  const data: unknown = { id: '1', email: 'test@example.com' };
  assertUser(data);
  // TypeScript now knows data is User
  console.log(data.id, data.email);
});
```

## Testing Async Code

### Promises and Async/Await

```typescript
// Test async functions
it('should handle async operations', async () => {
  const result = await asyncFunction();
  expect(result).toBeDefined();
});

// Test promise rejections
it('should throw on error', async () => {
  await expect(asyncFunction()).rejects.toThrow('Error message');
});

// Test with timeouts
it('should complete within timeout', async () => {
  await Promise.race([
    asyncFunction(),
    new Promise((_, reject) =>
      setTimeout(() => reject(new Error('Timeout')), 1000)
    ),
  ]);
});
```

### Mocking Async Functions

```typescript
// Mock async function
const mockFetch = jest.fn();

mockFetch.mockResolvedValue({
  json: async () => ({ data: 'test' }),
});

// Test error cases
mockFetch.mockRejectedValue(new Error('Network error'));

// Test with delays
mockFetch.mockImplementation(
  () =>
    new Promise(resolve =>
      setTimeout(() => resolve({ data: 'delayed' }), 100)
    )
);
```

## Property-Based Testing

### Using Libraries

```typescript
// Use fast-check for property-based testing
import fc from 'fast-check';

describe('Math utilities', () => {
  it('addition is commutative', () => {
    fc.assert(
      fc.property(fc.integer(), fc.integer(), (a, b) => {
        expect(add(a, b)).toBe(add(b, a));
      })
    );
  });
  
  it('string reverse is involutive', () => {
    fc.assert(
      fc.property(fc.string(), (str) => {
        expect(reverse(reverse(str))).toBe(str);
      })
    );
  });
});
```

## Integration Testing

### Database Testing

```typescript
// Test with real database (in-memory or test DB)
describe('UserRepository Integration', () => {
  let db: Database;
  
  beforeAll(async () => {
    db = await createTestDatabase();
  });
  
  afterAll(async () => {
    await db.close();
  });
  
  beforeEach(async () => {
    await db.clear();
  });
  
  it('should persist and retrieve user', async () => {
    const repo = new PostgresUserRepository(db);
    const user = await repo.save(UserFactory.create());
    
    const retrieved = await repo.findById(user.id);
    
    expect(retrieved).toEqual(user);
  });
});
```

### API Testing

```typescript
// Test HTTP endpoints
describe('User API', () => {
  let app: Express;
  
  beforeAll(() => {
    app = createTestApp();
  });
  
  it('should create user via POST', async () => {
    const response = await request(app)
      .post('/api/users')
      .send({ email: 'test@example.com', name: 'Test' })
      .expect(201);
    
    expect(response.body).toMatchObject({
      id: expect.any(String),
      email: 'test@example.com',
    });
  });
});
```

## Best Practices

1. **Arrange-Act-Assert (AAA)**:
   - Arrange: Set up test data
   - Act: Execute code under test
   - Assert: Verify results

2. **One assertion per test** (when possible):
   - Makes failures clear
   - Easier to debug

3. **Test behavior, not implementation**:
   - Focus on what, not how
   - Tests become more maintainable

4. **Use descriptive test names**:
   - Should describe what is being tested
   - Use format: "should [expected behavior] when [condition]"

5. **Keep tests independent**:
   - Each test should be able to run in isolation
   - Use beforeEach/afterEach for setup/teardown

6. **Mock external dependencies**:
   - Don't test third-party libraries
   - Mock network calls, file system, etc.
