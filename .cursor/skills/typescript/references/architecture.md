# TypeScript Architecture Patterns

## Layered Architecture

### Classic Three-Layer Pattern

```
┌─────────────────────────────────┐
│      Presentation Layer         │  (Controllers, Routes)
├─────────────────────────────────┤
│      Business Logic Layer       │  (Services, Use Cases)
├─────────────────────────────────┤
│      Data Access Layer          │  (Repositories, DAOs)
└─────────────────────────────────┘
```

```typescript
// Data Access Layer
interface UserRepository {
  findById(id: string): Promise<User | null>;
  save(user: User): Promise<User>;
}

class PostgresUserRepository implements UserRepository {
  async findById(id: string): Promise<User | null> {
    // Database implementation
  }
}

// Business Logic Layer
class UserService {
  constructor(private readonly userRepo: UserRepository) {}
  
  async getUser(id: string): Promise<User> {
    const user = await this.userRepo.findById(id);
    if (!user) {
      throw new NotFoundError('User');
    }
    return user;
  }
}

// Presentation Layer
class UserController {
  constructor(private readonly userService: UserService) {}
  
  async handleGetUser(req: Request, res: Response): Promise<void> {
    try {
      const user = await this.userService.getUser(req.params.id);
      res.json(user);
    } catch (error) {
      if (error instanceof NotFoundError) {
        res.status(404).json({ error: error.message });
      } else {
        res.status(500).json({ error: 'Internal server error' });
      }
    }
  }
}
```

## Domain-Driven Design (DDD)

### Entities and Value Objects

```typescript
// Value Object - Immutable, compared by value
class Email {
  private readonly value: string;
  
  constructor(email: string) {
    if (!this.isValid(email)) {
      throw new ValidationError('Invalid email format');
    }
    this.value = email.toLowerCase();
  }
  
  private isValid(email: string): boolean {
    return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email);
  }
  
  equals(other: Email): boolean {
    return this.value === other.value;
  }
  
  toString(): string {
    return this.value;
  }
}

// Entity - Has identity, compared by ID
class User {
  constructor(
    public readonly id: string,
    private email: Email,
    private name: string
  ) {}
  
  updateEmail(newEmail: Email): void {
    if (this.email.equals(newEmail)) {
      return; // No change
    }
    this.email = newEmail;
  }
  
  getEmail(): Email {
    return this.email;
  }
}
```

### Aggregates and Repositories

```typescript
// Aggregate Root
class Order {
  private constructor(
    public readonly id: string,
    private items: OrderItem[],
    private status: OrderStatus
  ) {}
  
  static create(id: string, items: OrderItem[]): Order {
    if (items.length === 0) {
      throw new ValidationError('Order must have at least one item');
    }
    return new Order(id, items, 'pending');
  }
  
  addItem(item: OrderItem): void {
    if (this.status !== 'pending') {
      throw new DomainError('Cannot modify completed order');
    }
    this.items.push(item);
  }
  
  calculateTotal(): number {
    return this.items.reduce((sum, item) => sum + item.total, 0);
  }
  
  complete(): void {
    if (this.status !== 'pending') {
      throw new DomainError('Order already completed');
    }
    this.status = 'completed';
  }
}

interface OrderRepository {
  findById(id: string): Promise<Order | null>;
  save(order: Order): Promise<void>;
}
```

## Hexagonal Architecture (Ports & Adapters)

### Defining Ports

```typescript
// Port - Interface defining what we need
interface UserStorage {
  save(user: User): Promise<void>;
  findById(id: string): Promise<User | null>;
}

interface EmailService {
  send(to: Email, subject: string, body: string): Promise<void>;
}

// Adapter - Implementation of port
class PostgresUserStorage implements UserStorage {
  async save(user: User): Promise<void> {
    // PostgreSQL implementation
  }
  
  async findById(id: string): Promise<User | null> {
    // PostgreSQL implementation
  }
}

class SendGridEmailService implements EmailService {
  async send(to: Email, subject: string, body: string): Promise<void> {
    // SendGrid API call
  }
}

// Core business logic depends only on ports
class UserRegistrationUseCase {
  constructor(
    private readonly userStorage: UserStorage,
    private readonly emailService: EmailService
  ) {}
  
  async execute(email: Email, name: string): Promise<User> {
    const user = User.create(email, name);
    await this.userStorage.save(user);
    await this.emailService.send(
      email,
      'Welcome',
      `Welcome ${name}!`
    );
    return user;
  }
}
```

## CQRS (Command Query Responsibility Segregation)

### Separating Reads and Writes

```typescript
// Commands - Mutations
interface CreateUserCommand {
  email: string;
  name: string;
}

interface UpdateUserCommand {
  userId: string;
  name?: string;
}

class CreateUserHandler {
  constructor(private readonly userRepo: UserRepository) {}
  
  async handle(command: CreateUserCommand): Promise<string> {
    const user = User.create(command.email, command.name);
    await this.userRepo.save(user);
    return user.id;
  }
}

// Queries - Read-only
interface GetUserQuery {
  userId: string;
}

interface UserQueryResult {
  id: string;
  email: string;
  name: string;
}

class GetUserQueryHandler {
  constructor(private readonly userRepo: UserRepository) {}
  
  async handle(query: GetUserQuery): Promise<UserQueryResult | null> {
    const user = await this.userRepo.findById(query.userId);
    if (!user) return null;
    
    return {
      id: user.id,
      email: user.email.toString(),
      name: user.name,
    };
  }
}
```

## Event Sourcing

### Event-Driven Architecture

```typescript
// Domain Event
interface DomainEvent {
  readonly type: string;
  readonly aggregateId: string;
  readonly occurredAt: Date;
}

class UserCreatedEvent implements DomainEvent {
  readonly type = 'UserCreated';
  constructor(
    public readonly aggregateId: string,
    public readonly email: string,
    public readonly name: string,
    public readonly occurredAt: Date = new Date()
  ) {}
}

// Event Store
interface EventStore {
  append(aggregateId: string, events: DomainEvent[]): Promise<void>;
  getEvents(aggregateId: string): Promise<DomainEvent[]>;
}

// Aggregate rebuilds from events
class User {
  private events: DomainEvent[] = [];
  
  static fromEvents(events: DomainEvent[]): User {
    const user = new User();
    events.forEach(event => user.apply(event));
    return user;
  }
  
  static create(id: string, email: Email, name: string): User {
    const user = new User(id, email, name);
    user.addEvent(new UserCreatedEvent(id, email.toString(), name));
    return user;
  }
  
  private apply(event: DomainEvent): void {
    // Rebuild state from events
    if (event instanceof UserCreatedEvent) {
      this.id = event.aggregateId;
      this.email = new Email(event.email);
      this.name = event.name;
    }
  }
}
```

## Dependency Injection Container

### Simple DI Container

```typescript
type Constructor<T = {}> = new (...args: any[]) => T;

class Container {
  private services = new Map<string, any>();
  private factories = new Map<string, (...args: any[]) => any>();
  
  register<T>(key: string, factory: () => T): void {
    this.factories.set(key, factory);
  }
  
  registerSingleton<T>(key: string, instance: T): void {
    this.services.set(key, instance);
  }
  
  resolve<T>(key: string): T {
    if (this.services.has(key)) {
      return this.services.get(key);
    }
    
    if (this.factories.has(key)) {
      const instance = this.factories.get(key)!();
      this.services.set(key, instance);
      return instance;
    }
    
    throw new Error(`Service ${key} not found`);
  }
}

// Usage
const container = new Container();
container.register('UserRepository', () => new PostgresUserRepository());
container.register('UserService', () => 
  new UserService(container.resolve('UserRepository'))
);
```

## Module Organization

### Feature-Based Modules

```
src/
├── features/
│   ├── authentication/
│   │   ├── domain/
│   │   │   ├── user.entity.ts
│   │   │   ├── user.repository.ts
│   │   │   └── auth.service.ts
│   │   ├── infrastructure/
│   │   │   └── postgres-user.repository.ts
│   │   └── presentation/
│   │       └── auth.controller.ts
│   └── orders/
│       └── ...
├── shared/
│   ├── domain/
│   │   ├── errors.ts
│   │   └── value-objects.ts
│   └── infrastructure/
│       └── event-bus.ts
└── infrastructure/
    ├── database/
    └── config/
```

### Barrel Exports Pattern

```typescript
// features/users/index.ts
export { User } from './domain/user.entity';
export { UserRepository } from './domain/user.repository';
export { UserService } from './domain/user.service';
export type { CreateUserDto, UpdateUserDto } from './domain/user.types';

// Usage throughout app
import { UserService, CreateUserDto } from '@/features/users';
```
