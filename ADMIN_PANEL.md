# Admin Panel Documentation

## Overview

The Rext Admin Panel provides comprehensive system administration capabilities including user management, request logging, database inspection, and system health monitoring.

## Access

### Default Admin Credentials
- **Email**: `admin@localhost.com`
- **Password**: `admin`

### Login URL
Navigate to `/admin/login` to access the admin panel.

## Features

### 1. Dashboard (`/admin`)
- **Overview Cards**: Display key metrics including total users, request logs, database tables, and system status
- **Recent Activity**: Shows the latest API requests with status codes and response times
- **Quick Actions**: Direct links to other admin sections

### 2. Request Logs (`/admin/logs`)
- **AG Grid Integration**: Advanced data table with sorting, filtering, and pagination
- **Filtering Options**:
  - HTTP Method (GET, POST, PUT, DELETE, PATCH)
  - Status Code (200, 201, 400, 401, 403, 404, 500)
  - Date Range (Start/End dates)
  - User ID
  - Page Size (10, 25, 50, 100 records)
- **Log Details**: Click on any log entry to view detailed information including request/response bodies and error messages
- **Real-time Updates**: Refresh button to fetch latest logs

### 3. User Management (`/admin/users`)
- **User List**: AG Grid displaying all users with email, creation date, and admin status
- **Search & Filter**: Search by email and filter by admin status
- **CRUD Operations**:
  - **Create**: Add new users with email, password, and admin privileges
  - **View**: Click on user to see detailed information
  - **Edit**: Modify user email, password, and admin status
  - **Delete**: Remove users (with confirmation dialog)
- **Admin Protection**: Admins cannot delete their own accounts

### 4. Database Browser (`/admin/database`)
- **Table Overview**: Visual cards showing all database tables with record counts
- **Record Inspection**: Click on any table to view its records in an AG Grid
- **Dynamic Columns**: Automatically generates columns based on table structure
- **Record Details**: Click on any record to view all field values
- **Pagination**: Navigate through large datasets efficiently

### 5. System Health (`/admin/health`)
- **Status Monitoring**: Real-time system status indicators
- **Performance Metrics**:
  - Request statistics (total, success rate, average response time, error rate)
  - User activity (total users, admin users, active users)
  - System resources (memory, CPU, disk usage)
- **Error Tracking**: Recent error logs with detailed error messages
- **System Information**: Application version, environment, database connection status
- **Auto-refresh**: Metrics update automatically every minute

## Technical Implementation

### Frontend Architecture
- **Vue 3 + TypeScript**: Modern reactive framework with type safety
- **AG Grid**: Enterprise-grade data grid for complex data display
- **Vue Router**: Client-side routing with authentication guards
- **Responsive Design**: Mobile-friendly interface with CSS Grid and Flexbox

### Backend Integration
- **RESTful API**: All admin operations use the existing admin API endpoints
- **JWT Authentication**: Secure admin authentication with token-based sessions
- **Real-time Data**: Live updates from the backend API
- **Error Handling**: Comprehensive error handling and user feedback

### Security Features
- **Admin-only Access**: All admin routes require admin authentication
- **Token Validation**: JWT tokens are validated on every request
- **Route Guards**: Automatic redirection to login for unauthenticated users
- **CSRF Protection**: Built-in protection against cross-site request forgery

## Usage Examples

### Managing Users
1. Navigate to `/admin/users`
2. Click "Create User" to add a new user
3. Fill in email, password, and admin status
4. Use the search bar to find specific users
5. Click the action buttons (👁️, ✏️, 🗑️) to view, edit, or delete users

### Monitoring System Health
1. Navigate to `/admin/health`
2. Review the status cards for overall system health
3. Check performance metrics for any anomalies
4. Review recent errors in the error log section
5. Use the refresh button to get latest data

### Analyzing Request Logs
1. Navigate to `/admin/logs`
2. Use filters to narrow down specific requests
3. Click on any log entry to see detailed information
4. Export data by copying from the grid
5. Monitor error rates and response times

### Database Inspection
1. Navigate to `/admin/database`
2. Click on any table card to view its records
3. Use the grid to sort and filter records
4. Click on any record to see all field values
5. Navigate through pages for large datasets

## Troubleshooting

### Common Issues

1. **Cannot Access Admin Panel**
   - Ensure you're using the correct admin credentials
   - Check that the backend server is running
   - Verify the admin user exists in the database

2. **Data Not Loading**
   - Check browser console for JavaScript errors
   - Verify API endpoints are accessible
   - Ensure admin token is valid

3. **AG Grid Not Displaying**
   - Check that AG Grid CSS is properly imported
   - Verify data format matches expected structure
   - Check for JavaScript errors in console

### Development Notes

- The admin panel uses the existing admin API endpoints
- All components are built with Vue 3 Composition API
- TypeScript provides type safety throughout the application
- AG Grid is configured for optimal performance with large datasets
- The interface is fully responsive and mobile-friendly

## Future Enhancements

- Real-time WebSocket updates for live monitoring
- Export functionality (CSV, JSON, Excel)
- Advanced analytics and reporting
- Role-based permissions for different admin levels
- Audit trail for admin actions
- System backup and restore functionality 