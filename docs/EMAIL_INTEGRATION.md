# Email Integration Guide

This guide explains how to configure and use email functionality in JHipster Rust applications. Email support enables user registration with activation, password reset, and other transactional emails.

## Overview

Email integration in JHipster Rust uses:

- **[Lettre](https://lettre.rs/)** - A mature Rust email library for SMTP transport
- **[Tera](https://tera.netlify.app/)** - A template engine (similar to Jinja2) for HTML email templates

### Supported Email Types

| Email Type         | Trigger                          | Template                |
| ------------------ | -------------------------------- | ----------------------- |
| Account Activation | User self-registration           | `activation.html`       |
| Password Reset     | Forgot password request          | `password-reset.html`   |
| Password Changed   | After successful password change | `password-changed.html` |
| Account Created    | Admin creates a user account     | `account-created.html`  |

## Configuration

### Environment Variables

Add the following environment variables to your `.env` file to configure email:

```env
# Email Configuration
# Set MAIL_ENABLED=true to enable email sending
MAIL_ENABLED=false

# SMTP Server Configuration
MAIL_HOST=localhost
MAIL_PORT=1025

# SMTP Authentication (optional - leave empty for local dev servers)
MAIL_USERNAME=
MAIL_PASSWORD=

# Sender Information
MAIL_FROM=noreply@yourdomain.com
MAIL_FROM_NAME=YourAppName

# Security Settings
# Use TLS for the connection (typically port 465)
MAIL_USE_TLS=false
# Use STARTTLS upgrade (typically port 587)
MAIL_USE_STARTTLS=false

# Base URL for links in emails (activation links, password reset links)
# Should match your frontend URL
MAIL_BASE_URL=http://localhost:9000
```

### Configuration Reference

| Variable            | Default                 | Description                                           |
| ------------------- | ----------------------- | ----------------------------------------------------- |
| `MAIL_ENABLED`      | `false`                 | Master switch to enable/disable email sending         |
| `MAIL_HOST`         | `localhost`             | SMTP server hostname                                  |
| `MAIL_PORT`         | `1025`                  | SMTP server port                                      |
| `MAIL_USERNAME`     | (empty)                 | SMTP authentication username (optional)               |
| `MAIL_PASSWORD`     | (empty)                 | SMTP authentication password (optional)               |
| `MAIL_FROM`         | `noreply@localhost`     | Sender email address                                  |
| `MAIL_FROM_NAME`    | App name                | Sender display name                                   |
| `MAIL_USE_TLS`      | `false`                 | Use TLS encryption (implicit TLS, typically port 465) |
| `MAIL_USE_STARTTLS` | `false`                 | Use STARTTLS upgrade (typically port 587)             |
| `MAIL_BASE_URL`     | `http://localhost:9000` | Base URL for links in emails                          |

## Behavior When Email is Disabled

When `MAIL_ENABLED=false` (the default):

1. The application starts normally without attempting SMTP connections
2. Email-related operations log warnings but don't fail
3. User registration creates **already-activated** users (no activation email needed)
4. Password reset requests are logged but no emails are sent
5. The application continues to function without email functionality

This allows development and testing without requiring an SMTP server.

## Local Development with MailHog

[MailHog](https://github.com/mailhog/MailHog) is a local SMTP server that captures emails for testing. It provides a web UI to view sent emails without actually delivering them.

### Option 1: Add MailHog to docker-compose.yml

Add the following service to your `docker-compose.yml`:

```yaml
services:
  # ... your existing services (mongodb, postgres, etc.)

  mailhog:
    image: mailhog/mailhog:v1.0.1
    container_name: ${APP_NAME:-app}-mailhog
    ports:
      - '1025:1025' # SMTP server
      - '8025:8025' # Web UI
```

### Option 2: Create docker-compose.override.yml

Create a separate override file for email testing:

```yaml
# docker-compose.override.yml
services:
  mailhog:
    image: mailhog/mailhog:v1.0.1
    container_name: ${APP_NAME:-app}-mailhog
    ports:
      - '1025:1025' # SMTP server
      - '8025:8025' # Web UI
```

### Option 3: Run MailHog Standalone

```bash
docker run -d --name mailhog -p 1025:1025 -p 8025:8025 mailhog/mailhog:v1.0.1
```

### Using MailHog

1. Start MailHog using one of the options above
2. Configure your `.env` file:

```env
MAIL_ENABLED=true
MAIL_HOST=localhost
MAIL_PORT=1025
MAIL_FROM=noreply@yourapp.local
MAIL_FROM_NAME=YourApp
MAIL_BASE_URL=http://localhost:9000
```

3. Start your application
4. Open MailHog Web UI at [http://localhost:8025](http://localhost:8025)
5. Trigger an email (register a user, request password reset, etc.)
6. View the captured email in MailHog

### Alternative: Mailpit

[Mailpit](https://github.com/axllent/mailpit) is a modern alternative to MailHog with additional features:

```yaml
services:
  mailpit:
    image: axllent/mailpit:latest
    container_name: ${APP_NAME:-app}-mailpit
    ports:
      - '1025:1025' # SMTP server
      - '8025:8025' # Web UI
```

## Production Configuration

For production, configure a real SMTP service:

### SendGrid

```env
MAIL_ENABLED=true
MAIL_HOST=smtp.sendgrid.net
MAIL_PORT=587
MAIL_USERNAME=apikey
MAIL_PASSWORD=your-sendgrid-api-key
MAIL_USE_STARTTLS=true
MAIL_FROM=noreply@yourdomain.com
MAIL_FROM_NAME=YourApp
MAIL_BASE_URL=https://yourdomain.com
```

### AWS SES

```env
MAIL_ENABLED=true
MAIL_HOST=email-smtp.us-east-1.amazonaws.com
MAIL_PORT=587
MAIL_USERNAME=your-ses-smtp-username
MAIL_PASSWORD=your-ses-smtp-password
MAIL_USE_STARTTLS=true
MAIL_FROM=noreply@yourdomain.com
MAIL_FROM_NAME=YourApp
MAIL_BASE_URL=https://yourdomain.com
```

### Mailgun

```env
MAIL_ENABLED=true
MAIL_HOST=smtp.mailgun.org
MAIL_PORT=587
MAIL_USERNAME=postmaster@your-mailgun-domain
MAIL_PASSWORD=your-mailgun-smtp-password
MAIL_USE_STARTTLS=true
MAIL_FROM=noreply@yourdomain.com
MAIL_FROM_NAME=YourApp
MAIL_BASE_URL=https://yourdomain.com
```

### Gmail (Development Only)

> **Warning**: Not recommended for production. Use an App Password, not your regular password.

```env
MAIL_ENABLED=true
MAIL_HOST=smtp.gmail.com
MAIL_PORT=587
MAIL_USERNAME=your-email@gmail.com
MAIL_PASSWORD=your-app-password
MAIL_USE_STARTTLS=true
MAIL_FROM=your-email@gmail.com
MAIL_FROM_NAME=YourApp
MAIL_BASE_URL=http://localhost:9000
```

## API Endpoints

When email is enabled, the following public endpoints are available:

### POST /api/register

Register a new user account. Creates an inactive user and sends an activation email.

**Request Body:**

```json
{
  "login": "johndoe",
  "email": "john@example.com",
  "password": "securePassword123",
  "langKey": "en"
}
```

**Response:** `201 Created`

**Errors:**

- `400 Bad Request` - Invalid password (must be 4-100 characters) or duplicate login/email

### GET /api/activate?key={activationKey}

Activate a user account using the key from the activation email.

**Response:** `200 OK`

**Errors:**

- `500 Internal Server Error` - Invalid or expired activation key

### POST /api/account/reset-password/init

Request a password reset. Sends a reset email if the account exists.

**Request Body:** Plain text email address

```
john@example.com
```

**Response:** `200 OK` (always, to prevent email enumeration)

### POST /api/account/reset-password/finish

Complete the password reset using the key from the reset email.

**Request Body:**

```json
{
  "key": "abc123resetkey",
  "newPassword": "newSecurePassword456"
}
```

**Response:** `200 OK`

**Errors:**

- `400 Bad Request` - Invalid password (must be 4-100 characters)
- `500 Internal Server Error` - Invalid or expired reset key (24-hour expiration)

## Email Templates

Email templates are located in `server/src/templates/email/` and use [Tera](https://tera.netlify.app/) syntax for variable interpolation.

### Template Variables

#### activation.html

| Variable               | Description                  |
| ---------------------- | ---------------------------- |
| `{{ user_name }}`      | User's first name or login   |
| `{{ activation_url }}` | Full activation URL with key |
| `{{ base_url }}`       | Base URL of the application  |

#### password-reset.html

| Variable          | Description                      |
| ----------------- | -------------------------------- |
| `{{ user_name }}` | User's first name or login       |
| `{{ reset_url }}` | Full password reset URL with key |
| `{{ base_url }}`  | Base URL of the application      |

#### password-changed.html

| Variable          | Description                 |
| ----------------- | --------------------------- |
| `{{ user_name }}` | User's first name or login  |
| `{{ base_url }}`  | Base URL of the application |

#### account-created.html

| Variable          | Description                 |
| ----------------- | --------------------------- |
| `{{ user_name }}` | User's first name or login  |
| `{{ login }}`     | User's login/username       |
| `{{ reset_url }}` | URL to set initial password |
| `{{ base_url }}`  | Base URL of the application |

### Customizing Templates

To customize email templates:

1. Locate the templates in `server/src/templates/email/`
2. Edit the HTML and CSS as needed
3. Use `{{ variable_name }}` for dynamic content
4. The templates are embedded at compile time, so rebuild after changes

Example customization:

```html
<!-- activation.html -->
<div class="header">
  <img src="{{ base_url }}/assets/logo.png" alt="Logo" />
  <h1>Welcome to Our Platform!</h1>
</div>
<div class="content">
  <p>Hi {{ user_name }},</p>
  <p>Thanks for signing up! Click below to activate your account:</p>
  <a href="{{ activation_url }}" class="button">Activate Now</a>
</div>
```

## Security Considerations

| Concern             | Mitigation                                       |
| ------------------- | ------------------------------------------------ |
| Email enumeration   | `/reset-password/init` always returns 200 OK     |
| Key guessing        | 20-character random alphanumeric keys            |
| Key expiration      | Reset and activation keys expire after 24 hours  |
| Password validation | 4-100 character requirement matching Spring Boot |
| HTML injection      | Tera auto-escapes variables by default           |
| Credential exposure | Passwords stored using Argon2 hashing            |

## Troubleshooting

### Emails Not Sending

1. **Check MAIL_ENABLED**: Ensure `MAIL_ENABLED=true` in your `.env`
2. **Check logs**: Look for email-related log messages
3. **Verify SMTP connection**: Test with a tool like `telnet`:
   ```bash
   telnet localhost 1025
   ```
4. **Check firewall**: Ensure the SMTP port is accessible

### Connection Errors

```
Failed to initialize email service: Failed to create SMTP relay
```

- Verify `MAIL_HOST` and `MAIL_PORT` are correct
- Check if the SMTP server is running
- Try with `MAIL_USE_TLS=false` for local development

### Authentication Errors

```
Failed to send email: Authentication failed
```

- Verify `MAIL_USERNAME` and `MAIL_PASSWORD`
- For Gmail, ensure you're using an App Password
- Check if your SMTP provider requires specific authentication methods

### Template Errors

```
Failed to render template activation.html
```

- Check template syntax for Tera compatibility
- Ensure all required variables are passed to the template
- Verify template files exist in `server/src/templates/email/`

## Architecture

```
┌────────────────────────────────────────────────────────────────┐
│                         Application                            │
├────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐     ┌──────────────┐     ┌──────────────────┐ │
│  │   Handler   │────▶│ EmailService │────▶│  SMTP Transport  │ │
│  │  (account)  │     │              │     │    (lettre)      │ │
│  └─────────────┘     └──────────────┘     └────────┬─────────┘ │
│                             │                      │           │
│                      ┌──────▼──────┐               │           │
│                      │    Tera     │               │           │
│                      │  Templates  │               │           │
│                      └─────────────┘               │           │
└────────────────────────────────────────────────────┼───────────┘
                                                     │
                                              ┌──────▼──────┐
                                              │ SMTP Server │
                                              │  (MailHog/  │
                                              │  SendGrid)  │
                                              └─────────────┘
```

## Compatibility

This implementation is designed to be compatible with:

- **Spring Boot JHipster** - Same API endpoints and request/response formats
- **JHipster Angular/React UI** - Works with standard JHipster frontends
- **OAuth2 note**: Email features are only available with JWT authentication (OAuth2 handles user management externally)
