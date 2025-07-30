# Session Management Implementation Plan

## Overview
This document outlines the implementation plan for adding session management to the Admin Panel (completing Phase 4.6). The feature will allow administrators to view active user sessions, see device information, and remotely log out users.

## Current System Analysis

### What We Have
- JWT-based authentication with 24-hour token expiry
- User agent capture in logging middleware (`backend/bridge/middleware/logging.rs:197`)
- Comprehensive admin user management interface
- Role-based permission system
- Audit logging for admin actions

### What We Need
- Persistent session tracking with device/IP information
- Session validation during token verification
- Session activity updates
- Admin UI for session management
- Remote session termination capability

## Technical Requirements

### Database Schema
**New Table: `user_sessions`**
```sql
CREATE TABLE user_sessions (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_token VARCHAR(97) NOT NULL UNIQUE, -- Argon2 hash of JWT token
    user_agent TEXT,
    ip_address VARCHAR(45), -- Supports both IPv4 and IPv6
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    last_activity TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL,
    is_active BOOLEAN DEFAULT true
);

CREATE INDEX idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX idx_user_sessions_token ON user_sessions(session_token);
CREATE INDEX idx_user_sessions_active ON user_sessions(is_active);
CREATE INDEX idx_user_sessions_expires ON user_sessions(expires_at);
```

### Session Lifecycle
1. **Session Creation**: When user logs in, create session record
2. **Session Validation**: Check session exists and is active during token validation
3. **Activity Tracking**: Update `last_activity` on each authenticated request
4. **Session Expiry**: Sessions expire 24 hours from last activity
5. **Remote Logout**: Admin can invalidate sessions by setting `is_active = false`

## Implementation Steps

### 1. Database Layer

#### 1.1 Create Migration
**File**: `migration/src/m20250101_000005_create_user_sessions.rs`
- Create `user_sessions` table with proper indexes
- Add foreign key to users table with CASCADE delete

#### 1.2 Create Entity Model
**File**: `backend/entity/models/user_sessions.rs`
- Sea-ORM entity with all required fields
- Relationships to users table
- Proper serialization for API responses

### 2. Backend Services

#### 2.1 Session Service
**File**: `backend/control/services/session_service.rs`

**Core Functions:**
```rust
impl SessionService {
    // Create new session on login
    async fn create_session(db: &DatabaseConnection, user_id: Uuid, user_agent: Option<String>, ip_address: Option<String>, jwt_token: &str) -> Result<UserSessionModel, AppError>
    
    // Validate session exists and is active
    async fn validate_session(db: &DatabaseConnection, token_hash: &str) -> Result<UserSessionModel, AppError>
    
    // Update session activity timestamp
    async fn update_session_activity(db: &DatabaseConnection, session_id: Uuid) -> Result<(), AppError>
    
    // Get active sessions for a user
    async fn get_user_sessions(db: &DatabaseConnection, user_id: Uuid) -> Result<Vec<UserSessionModel>, AppError>
    
    // Invalidate session (admin logout)
    async fn invalidate_session(db: &DatabaseConnection, session_id: Uuid) -> Result<(), AppError>
    
    // Cleanup expired sessions (background task)
    async fn cleanup_expired_sessions(db: &DatabaseConnection) -> Result<u64, AppError>
    
    // Generate Argon2 hash of JWT token for secure storage
    // Uses same Argon2 config as UserService for consistency
    fn generate_session_token_hash(jwt_token: &str) -> Result<String, AppError>
    
    // Verify JWT token against stored Argon2 hash
    fn verify_session_token(jwt_token: &str, stored_hash: &str) -> Result<bool, AppError>
}
```

**Note**: The SessionService will leverage the existing Argon2 configuration from `UserService` to ensure consistent hashing parameters across the application.

#### 2.2 Update JWT Claims
**File**: `backend/infrastructure/jwt_claims.rs`
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,    // user id
    pub exp: usize,     // expiration time
    pub session_id: String, // session UUID for tracking
}
```

#### 2.3 Update Auth Service
**File**: `backend/control/services/auth_service.rs`

**Changes to `authenticate_user`:**
1. Generate JWT token with session_id
2. Create session record with Argon2 hash of JWT token, user agent, and IP
3. Return token with session tracking

#### 2.4 Update Token Service
**File**: `backend/control/services/token_service.rs`

**Changes to `validate_token`:**
1. Validate JWT token structure and expiry as before
2. Generate Argon2 hash of the JWT token
3. Look up session by token hash and validate it's active and not expired
4. Return user_id and session_id for middleware

### 3. Middleware Updates

#### 3.1 Update Auth Middleware
**File**: `backend/bridge/middleware/auth.rs`

**Add session activity tracking:**
```rust
// After successful token validation
if let Some(session_id) = session_id {
    // Update session activity (fire and forget)
    let db_clone = db.clone();
    tokio::spawn(async move {
        let _ = SessionService::update_session_activity(&db_clone, session_id).await;
    });
}
```

### 4. Admin API Endpoints

#### 4.1 Add Session Management to Admin Service
**File**: `backend/control/services/admin_service.rs`

**New Functions:**
```rust
impl AdminService {
    // Get sessions for a specific user
    async fn get_user_sessions(db: &DatabaseConnection, user_id: Uuid) -> Result<Vec<SessionResponse>, AppError>
    
    // Invalidate a specific session
    async fn invalidate_user_session(db: &DatabaseConnection, session_id: Uuid) -> Result<(), AppError>
    
    // Invalidate all sessions for a user
    async fn invalidate_all_user_sessions(db: &DatabaseConnection, user_id: Uuid) -> Result<u64, AppError>
}
```

#### 4.2 Add Session Endpoints
**File**: `backend/bridge/handlers/admin.rs`

**New Endpoints:**
- `GET /api/v1/admin/users/{user_id}/sessions` - Get user sessions
- `DELETE /api/v1/admin/sessions/{session_id}` - Invalidate specific session
- `DELETE /api/v1/admin/users/{user_id}/sessions` - Invalidate all user sessions

#### 4.3 Add Types
**File**: `backend/bridge/types/admin.rs`

```rust
#[derive(Serialize, ToSchema)]
pub struct SessionResponse {
    pub id: String,
    pub user_id: String,
    pub device_info: String,      // Parsed user agent
    pub ip_address: Option<String>,
    pub created_at: String,
    pub last_activity: String,
    pub expires_at: String,
    pub is_current: bool,         // If this is the current session
}

#[derive(Deserialize, ToSchema)]
pub struct InvalidateSessionRequest {
    pub session_id: String,
}
```

### 5. Frontend Implementation

#### 5.1 Update Admin Users View
**File**: `frontend/src/appearance/views/AdminUsersView.vue`

**Add Session Management Tab:**
- New tab in user details modal: "Sessions"
- Show active sessions with device info, IP, last activity
- "Log Out" button for each session
- "Log Out All Sessions" button

#### 5.2 User Agent Parsing
**Frontend JavaScript function:**
```javascript
function parseUserAgent(userAgent) {
    // Parse user agent string to friendly device name
    // Example: "Mozilla/5.0 (Windows NT 10.0; Win64; x64)..." → "Chrome on Windows"
    // Use a library like 'ua-parser-js' or implement custom parsing
}
```

#### 5.3 Session Management Component
**File**: `frontend/src/appearance/components/UserSessionsComponent.vue`

**Features:**
- List of active sessions
- Device type icons (desktop, mobile, tablet)
- Last activity timestamps
- Session actions (logout)
- Confirmation dialogs

#### 5.4 Error Handling
**Update API client to handle session invalidation:**
- Catch 401 errors with session invalidation message
- Automatically logout user and redirect to login
- Clear stored JWT token

### 6. Security Considerations

#### 6.1 Token Hashing
- Store Argon2 hash of JWT token in database (not full token)
- Uses same hashing approach as passwords for consistency
- Prevents token exposure if database is compromised
- More secure than SHA-256 against brute force attacks

#### 6.2 Session Validation Performance
- Use database indexes for fast session lookups
- Consider Redis cache for high-traffic scenarios (future enhancement)

#### 6.3 Audit Logging
- Log session creation, invalidation, and cleanup
- Track admin actions for session management

### 7. Testing Strategy

#### 7.1 Backend Tests
- Session CRUD operations
- Token validation with session checks
- Session expiry and cleanup
- Admin session management endpoints

#### 7.2 Frontend Tests
- Session display and parsing
- Remote logout functionality
- Error handling for invalid sessions

#### 7.3 Integration Tests
- Complete login → session creation → validation flow
- Admin session management workflow
- Session expiry and cleanup

## Implementation Order

1. **Database Migration** - Create user_sessions table
2. **Entity Model** - Add UserSession entity
3. **Session Service** - Core session management logic
4. **JWT Claims Update** - Add session_id to claims
5. **Auth Service Update** - Create sessions on login
6. **Token Service Update** - Validate sessions
7. **Middleware Update** - Track session activity
8. **Admin Endpoints** - Session management API
9. **Frontend UI** - Session management interface
10. **Error Handling** - Invalid session handling
11. **Testing** - Comprehensive test coverage

## Database Migration Strategy

### Migration File Structure
```rust
// m20250101_000005_create_user_sessions.rs
#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create user_sessions table
        // Add indexes for performance
        // Set up foreign key constraints
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop user_sessions table
    }
}
```

## Frontend Integration Points

### Existing Admin Interface
- **AdminUsersView.vue**: Add session management to user details
- **User action buttons**: Add "Manage Sessions" option
- **User grid columns**: Add "Active Sessions" count column

### New Components
- **SessionsList.vue**: Display user sessions with device info
- **SessionItem.vue**: Individual session with logout action
- **DeviceIcon.vue**: Display appropriate device icon

## Error Handling Strategy

### Backend Error Scenarios
1. **Invalid Session**: Return 401 with specific error code
2. **Expired Session**: Return 401 with expiry message
3. **Session Not Found**: Return 401 with session error
4. **Database Errors**: Log and return 500

### Frontend Error Handling
1. **Session Invalidation**: Auto-logout and redirect
2. **Network Errors**: Show retry option
3. **Permission Errors**: Show access denied message

## Performance Considerations

### Database Optimization
- Index on `session_token` for fast lookups
- Index on `user_id` for user session queries
- Index on `expires_at` for cleanup operations
- Composite index on `(user_id, is_active)` for active session queries

### Cleanup Strategy
- Background task to remove expired sessions
- Run cleanup every hour to prevent table bloat
- Archive old sessions for audit purposes (optional)

## Future Enhancements

### Phase 1 (Current Implementation)
- Basic session tracking and management
- Admin remote logout capability
- Device information display

### Phase 2 (Future)
- Redis cache for session validation
- WebSocket notifications for session changes
- Session analytics and reporting
- Concurrent session limits per user
- Remember me functionality with refresh tokens

## Conclusion

This implementation plan provides a comprehensive approach to adding session management to the Admin Panel. The solution balances security, performance, and usability while maintaining the existing architecture patterns.

The session management feature will give administrators complete visibility and control over user sessions, enhancing the platform's security capabilities and providing a professional admin experience similar to enterprise systems.