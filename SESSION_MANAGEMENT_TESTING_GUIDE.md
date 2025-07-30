# Session Management Testing Guide

## Prerequisites

Before testing, ensure you have:

1. **Run the database migration:**
   ```bash
   sea-orm-cli migrate up
   ```

2. **Start the backend server:**
   ```bash
   cargo run
   ```

3. **Start the frontend development server:**
   ```bash
   cd frontend && npm run dev
   ```

4. **Verify the migration was successful:**
   - Check that the `user_sessions` table exists in your database
   - The table should have columns: id, user_id, session_token, user_agent, ip_address, created_at, last_activity, expires_at, is_active

## Testing Steps

### 1. Backend API Testing

#### 1.1 Test User Authentication Creates Sessions
1. **Register/Login as a regular user** via `POST /api/v1/auth/login`
2. **Verify JWT token** contains session_id in claims
3. **Check database** - confirm a new record exists in `user_sessions` table
4. **Verify session fields**:
   - `session_token` should match the session_id from JWT
   - `user_agent` should be populated if sent in request
   - `ip_address` should be captured
   - `is_active` should be true
   - `expires_at` should be 24 hours from creation

#### 1.2 Test Admin Authentication Creates Sessions
1. **Login as admin** via `POST /api/v1/admin/login`
2. **Verify JWT token** contains session_id in claims
3. **Check database** - confirm admin session is created

#### 1.3 Test Session Validation
1. **Make authenticated request** with valid JWT
2. **Verify request succeeds** and session `last_activity` is updated
3. **Invalidate session** in database (set `is_active = false`)
4. **Make another request** with same JWT
5. **Verify request fails** with 401 and session invalidation message

#### 1.4 Test Session Management Endpoints

**Get User Sessions:**
```bash
curl -H "Authorization: Bearer <admin_token>" \
  http://localhost:3000/api/v1/admin/users/<user_id>/sessions
```

**Invalidate Specific Session:**
```bash
curl -X DELETE -H "Authorization: Bearer <admin_token>" \
  http://localhost:3000/api/v1/admin/sessions/<session_id>
```

**Invalidate All User Sessions:**
```bash
curl -X DELETE -H "Authorization: Bearer <admin_token>" \
  http://localhost:3000/api/v1/admin/users/<user_id>/sessions
```

### 2. Frontend Testing

#### 2.1 Test Admin Session Management UI

1. **Login to admin panel** at `http://localhost:5173/admin/login`
2. **Navigate to Users section**
3. **Click on a user** to open user details modal
4. **Click "Manage Sessions" button**
5. **Verify session modal opens** with:
   - List of active sessions
   - Device type icons (📱 for mobile, 💻 for desktop)
   - Parsed device names (e.g., "Chrome Browser", "Mobile Safari")
   - IP addresses
   - Last activity timestamps
   - Session creation dates

#### 2.2 Test Session Actions

1. **Test individual session logout:**
   - Click "🚪 Log Out" on a specific session
   - Verify session is removed from list
   - Verify session is marked inactive in database

2. **Test logout all sessions:**
   - Click "🚪 Log Out All Sessions"
   - Verify all sessions are removed from list
   - Verify all user sessions are marked inactive in database

3. **Test refresh functionality:**
   - Click "🔄 Refresh" button
   - Verify sessions list is updated

#### 2.3 Test Session Invalidation Handling

1. **Login as a user** in one browser/tab
2. **Login to admin** in another browser/tab
3. **Invalidate the user's session** through admin panel
4. **Make a request as the user** (navigate to a protected page)
5. **Verify user is automatically logged out** and redirected to login page

### 3. Multi-Device Testing

#### 3.1 Test Multiple Active Sessions

1. **Login from different browsers:**
   - Chrome desktop
   - Firefox desktop  
   - Mobile browser (or simulate with dev tools)
2. **Verify all sessions appear** in admin panel with different device info
3. **Test session activity tracking:**
   - Make requests from different browsers
   - Verify "last activity" timestamps update correctly

#### 3.2 Test Device Identification

1. **Test different user agents:**
   ```bash
   # Chrome desktop
   curl -H "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"
   
   # Mobile Safari
   curl -H "User-Agent: Mozilla/5.0 (iPhone; CPU iPhone OS 14_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0 Mobile/15E148 Safari/604.1"
   
   # Firefox
   curl -H "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:89.0) Gecko/20100101 Firefox/89.0"
   ```

2. **Verify device parsing:**
   - Chrome shows as "Chrome Browser" with 💻 icon
   - Mobile Safari shows as "Mobile Safari" with 📱 icon
   - Firefox shows as "Firefox Browser" with 💻 icon

### 4. Security Testing

#### 4.1 Test Session Expiry

1. **Create a session**
2. **Manually update database** to set `expires_at` to past timestamp
3. **Make authenticated request**
4. **Verify request fails** with session expired error

#### 4.2 Test Session Isolation

1. **Create sessions for multiple users**
2. **Verify user A cannot access user B's sessions**
3. **Test admin can access all user sessions**

#### 4.3 Test Token Validation

1. **Test with invalid JWT token**
2. **Test with malformed session_id in JWT**
3. **Test with session_id that doesn't exist in database**
4. **Verify all cases return appropriate 401 errors**

### 5. Performance Testing

#### 5.1 Test Session Activity Updates

1. **Make rapid requests** (e.g., 10 requests per second)
2. **Verify session activity updates** don't block request processing
3. **Check database** for reasonable update frequency (not every request)

#### 5.2 Test Session Cleanup

1. **Create multiple expired sessions** in database
2. **Run session cleanup** (this might be automated or manual)
3. **Verify expired sessions are removed**

### 6. Error Handling Testing

#### 6.1 Test Database Connection Issues

1. **Simulate database disconnection**
2. **Verify graceful error handling**
3. **Test recovery when database reconnects**

#### 6.2 Test Invalid Session Data

1. **Insert invalid session data** in database
2. **Test session validation handles gracefully**
3. **Verify appropriate error messages**

## Expected Results

### ✅ Success Criteria

- [ ] Users can login and sessions are created automatically
- [ ] JWT tokens contain session_id in claims
- [ ] Session validation works correctly for authenticated requests
- [ ] Session activity timestamps update on each request
- [ ] Admin can view all user sessions with device information
- [ ] Admin can invalidate individual sessions
- [ ] Admin can invalidate all user sessions
- [ ] Users are automatically logged out when sessions are invalidated
- [ ] Device types are correctly identified and displayed
- [ ] User agent strings are parsed into friendly names
- [ ] IP addresses are captured and displayed
- [ ] Session expiry works correctly (24 hours)
- [ ] Error handling redirects users to appropriate login pages
- [ ] Multiple concurrent sessions work correctly
- [ ] Session data is secure and isolated between users

### 🚨 Common Issues to Check

1. **Migration not run:** Session functionality won't work
2. **CORS issues:** Frontend can't communicate with backend
3. **Token storage:** Ensure tokens are properly stored in localStorage
4. **Time zone issues:** Check timestamp handling across different time zones
5. **Browser compatibility:** Test in different browsers for user agent parsing
6. **Mobile responsiveness:** Verify session modal works on mobile devices

### 📝 Testing Checklist

- [ ] Database migration completed successfully
- [ ] Backend session creation works
- [ ] Frontend session display works
- [ ] Session invalidation works
- [ ] Error handling redirects work
- [ ] Multi-device sessions work
- [ ] Device identification works
- [ ] Admin permissions work correctly
- [ ] Session security is maintained
- [ ] Performance is acceptable

## Troubleshooting

### Issue: Sessions not created
- Check database migration ran successfully
- Verify user_sessions table exists
- Check backend logs for errors
- Ensure JWT claims include session_id

### Issue: Session invalidation not working
- Check TokenService is using session validation
- Verify middleware is updated to use new token service
- Check session_id parsing in JWT claims

### Issue: Frontend not showing sessions
- Check API endpoints are accessible
- Verify admin authentication is working
- Check browser network tab for API errors
- Ensure session handler is imported correctly

### Issue: Users not automatically logged out
- Verify session handler is wrapping API calls
- Check error response format from backend
- Ensure handleSessionInvalidation function is called
- Verify localStorage tokens are cleared

This testing guide ensures comprehensive validation of the entire session management system!