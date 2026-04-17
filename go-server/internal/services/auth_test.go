package services

import (
	"context"
	"os"
	"regexp"
	"testing"
	"time"

	"github.com/DATA-DOG/go-sqlmock"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/models"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/pkg/logger"
	"github.com/stretchr/testify/assert"
	"golang.org/x/crypto/bcrypt"
	"gorm.io/driver/postgres"
	"gorm.io/gorm"
)

func TestMain(m *testing.M) {
	logger.Init()
	m.Run()
}

func setupAuthTestDB(t *testing.T) (*gorm.DB, sqlmock.Sqlmock) {
	sqlDB, mock, err := sqlmock.New()
	if err != nil {
		t.Fatalf("failed to create sqlmock: %v", err)
	}

	dialector := postgres.New(postgres.Config{
		Conn: sqlDB,
	})
	db, err := gorm.Open(dialector, &gorm.Config{
		SkipDefaultTransaction: true,
	})
	if err != nil {
		t.Fatalf("failed to open gorm db: %v", err)
	}

	return db, mock
}

func TestNewAuthService_DefaultSecret(t *testing.T) {
	db, _ := setupAuthTestDB(t)

	origEnv := os.Getenv("JWT_SECRET")
	origGoEnv := os.Getenv("GO_ENV")
	defer func() {
		os.Setenv("JWT_SECRET", origEnv)
		os.Setenv("GO_ENV", origGoEnv)
	}()

	os.Unsetenv("JWT_SECRET")
	os.Unsetenv("GO_ENV")

	svc := NewAuthService(db)
	assert.NotNil(t, svc)
	assert.NotNil(t, svc.jwtKey)
}

func TestNewAuthService_ProductionNoSecret(t *testing.T) {
	db, _ := setupAuthTestDB(t)

	origEnv := os.Getenv("JWT_SECRET")
	origGoEnv := os.Getenv("GO_ENV")
	defer func() {
		os.Setenv("JWT_SECRET", origEnv)
		os.Setenv("GO_ENV", origGoEnv)
	}()

	os.Unsetenv("JWT_SECRET")
	os.Setenv("GO_ENV", "production")

	defer func() {
		if r := recover(); r == nil {
			t.Errorf("expected panic in production without JWT_SECRET")
		}
	}()

	NewAuthService(db)
}

func TestNewAuthService_SecretTooShort(t *testing.T) {
	db, _ := setupAuthTestDB(t)

	origEnv := os.Getenv("JWT_SECRET")
	defer os.Setenv("JWT_SECRET", origEnv)

	os.Setenv("JWT_SECRET", "short")

	defer func() {
		if r := recover(); r == nil {
			t.Errorf("expected panic with short JWT_SECRET")
		}
	}()

	NewAuthService(db)
}

func TestRegister_Success(t *testing.T) {
	db, mock := setupAuthTestDB(t)

	origEnv := os.Getenv("JWT_SECRET")
	defer os.Setenv("JWT_SECRET", origEnv)
	os.Setenv("JWT_SECRET", "test-secret-at-least-32-characters-long")

	svc := NewAuthService(db)

	mock.ExpectQuery(regexp.QuoteMeta(`SELECT * FROM "users" WHERE username = $1 ORDER BY "users"."id" LIMIT $2`)).
		WithArgs("testuser", 1).
		WillReturnError(gorm.ErrRecordNotFound)

	// No transaction because SkipDefaultTransaction: true
	// INSERT with RETURNING is a Query, not Exec
	mock.ExpectQuery(`INSERT INTO "users"`).
		WithArgs("testuser", sqlmock.AnyArg(), "guest", sqlmock.AnyArg(), sqlmock.AnyArg()).
		WillReturnRows(sqlmock.NewRows([]string{"id"}).AddRow("test-user-id"))

	user, err := svc.Register(context.Background(), "testuser", "password123")

	assert.NoError(t, err)
	assert.NotNil(t, user)
	assert.Equal(t, "testuser", user.Username)
	assert.Equal(t, "guest", user.Role)
	assert.NoError(t, bcrypt.CompareHashAndPassword([]byte(user.PasswordHash), []byte("password123")))
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestRegister_ValidationError(t *testing.T) {
	tests := []struct {
		name     string
		username string
		password string
		errMsg   string
	}{
		{"empty username", "", "password123", "username and password are required"},
		{"empty password", "testuser", "", "username and password are required"},
		{"short password", "testuser", "12345", "password must be at least 6 characters"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			db, _ := setupAuthTestDB(t)

			origEnv := os.Getenv("JWT_SECRET")
			defer os.Setenv("JWT_SECRET", origEnv)
			os.Setenv("JWT_SECRET", "test-secret-at-least-32-characters-long")

			svc := NewAuthService(db)

			user, err := svc.Register(context.Background(), tt.username, tt.password)

			assert.Error(t, err)
			assert.Equal(t, tt.errMsg, err.Error())
			assert.Nil(t, user)
		})
	}
}

func TestRegister_DuplicateUsername(t *testing.T) {
	db, mock := setupAuthTestDB(t)

	origEnv := os.Getenv("JWT_SECRET")
	defer os.Setenv("JWT_SECRET", origEnv)
	os.Setenv("JWT_SECRET", "test-secret-at-least-32-characters-long")

	svc := NewAuthService(db)

	existingUser := models.User{
		ID:       "existing-id",
		Username: "existinguser",
		Role:     "guest",
	}
	rows := sqlmock.NewRows([]string{"id", "username", "password_hash", "role", "created_at", "updated_at"}).
		AddRow(existingUser.ID, existingUser.Username, "hash", existingUser.Role, time.Now(), time.Now())

	mock.ExpectQuery(regexp.QuoteMeta(`SELECT * FROM "users" WHERE username = $1 ORDER BY "users"."id" LIMIT $2`)).
		WithArgs("existinguser", 1).
		WillReturnRows(rows)

	user, err := svc.Register(context.Background(), "existinguser", "password123")

	assert.Error(t, err)
	assert.Equal(t, "username already exists", err.Error())
	assert.Nil(t, user)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestGenesisAdmin_Success(t *testing.T) {
	db, mock := setupAuthTestDB(t)

	origEnv := os.Getenv("JWT_SECRET")
	defer os.Setenv("JWT_SECRET", origEnv)
	os.Setenv("JWT_SECRET", "test-secret-at-least-32-characters-long")

	svc := NewAuthService(db)

	mock.ExpectBegin()
	mock.ExpectQuery(regexp.QuoteMeta(`SELECT count(*) FROM "users" WHERE role = $1`)).
		WithArgs("admin").
		WillReturnRows(sqlmock.NewRows([]string{"count"}).AddRow(0))

	mock.ExpectQuery(`INSERT INTO "users"`).
		WithArgs("adminuser", sqlmock.AnyArg(), "admin", sqlmock.AnyArg(), sqlmock.AnyArg()).
		WillReturnRows(sqlmock.NewRows([]string{"id"}).AddRow("test-admin-id"))
	mock.ExpectCommit()

	user, err := svc.GenesisAdmin(context.Background(), "adminuser", "adminpass123")

	assert.NoError(t, err)
	assert.NotNil(t, user)
	assert.Equal(t, "adminuser", user.Username)
	assert.Equal(t, "admin", user.Role)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestGenesisAdmin_Sealed(t *testing.T) {
	db, mock := setupAuthTestDB(t)

	origEnv := os.Getenv("JWT_SECRET")
	defer os.Setenv("JWT_SECRET", origEnv)
	os.Setenv("JWT_SECRET", "test-secret-at-least-32-characters-long")

	svc := NewAuthService(db)

	mock.ExpectBegin()
	mock.ExpectQuery(regexp.QuoteMeta(`SELECT count(*) FROM "users" WHERE role = $1`)).
		WithArgs("admin").
		WillReturnRows(sqlmock.NewRows([]string{"count"}).AddRow(1))
	mock.ExpectRollback()

	user, err := svc.GenesisAdmin(context.Background(), "adminuser", "adminpass123")

	assert.Error(t, err)
	assert.Equal(t, ErrGenesisSealed, err)
	assert.Nil(t, user)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestGenesisAdmin_ValidationError(t *testing.T) {
	tests := []struct {
		name     string
		username string
		password string
	}{
		{"empty username", "", "password123"},
		{"empty password", "admin", ""},
		{"short password", "admin", "12345"},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			db, mock := setupAuthTestDB(t)

			origEnv := os.Getenv("JWT_SECRET")
			defer os.Setenv("JWT_SECRET", origEnv)
			os.Setenv("JWT_SECRET", "test-secret-at-least-32-characters-long")

			svc := NewAuthService(db)

			mock.ExpectBegin()
			mock.ExpectQuery(regexp.QuoteMeta(`SELECT count(*) FROM "users" WHERE role = $1`)).
				WithArgs("admin").
				WillReturnRows(sqlmock.NewRows([]string{"count"}).AddRow(0))
			mock.ExpectRollback()

			user, err := svc.GenesisAdmin(context.Background(), tt.username, tt.password)

			assert.Error(t, err)
			assert.Nil(t, user)
			assert.NoError(t, mock.ExpectationsWereMet())
		})
	}
}

func TestIsGenesisSealed(t *testing.T) {
	tests := []struct {
		name       string
		adminCount int64
		expected   bool
	}{
		{"sealed", 1, true},
		{"not sealed", 0, false},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			db, mock := setupAuthTestDB(t)

			origEnv := os.Getenv("JWT_SECRET")
			defer os.Setenv("JWT_SECRET", origEnv)
			os.Setenv("JWT_SECRET", "test-secret-at-least-32-characters-long")

			svc := NewAuthService(db)

			mock.ExpectQuery(regexp.QuoteMeta(`SELECT count(*) FROM "users" WHERE role = $1`)).
				WithArgs("admin").
				WillReturnRows(sqlmock.NewRows([]string{"count"}).AddRow(tt.adminCount))

			result := svc.IsGenesisSealed(context.Background())

			assert.Equal(t, tt.expected, result)
			assert.NoError(t, mock.ExpectationsWereMet())
		})
	}
}

func TestLogin_Success(t *testing.T) {
	db, mock := setupAuthTestDB(t)

	origEnv := os.Getenv("JWT_SECRET")
	defer os.Setenv("JWT_SECRET", origEnv)
	os.Setenv("JWT_SECRET", "test-secret-at-least-32-characters-long")

	svc := NewAuthService(db)

	password := "password123"
	hash, _ := bcrypt.GenerateFromPassword([]byte(password), bcrypt.DefaultCost)

	userID := "user-123"
	rows := sqlmock.NewRows([]string{"id", "username", "password_hash", "role", "created_at", "updated_at"}).
		AddRow(userID, "testuser", string(hash), "admin", time.Now(), time.Now())

	mock.ExpectQuery(regexp.QuoteMeta(`SELECT * FROM "users" WHERE username = $1 ORDER BY "users"."id" LIMIT $2`)).
		WithArgs("testuser", 1).
		WillReturnRows(rows)

	accessToken, refreshToken, user, err := svc.Login(context.Background(), "testuser", password)

	assert.NoError(t, err)
	assert.NotNil(t, user)
	assert.NotEmpty(t, accessToken)
	assert.NotEmpty(t, refreshToken)
	assert.Equal(t, userID, user.ID)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestLogin_InvalidUsername(t *testing.T) {
	db, mock := setupAuthTestDB(t)

	origEnv := os.Getenv("JWT_SECRET")
	defer os.Setenv("JWT_SECRET", origEnv)
	os.Setenv("JWT_SECRET", "test-secret-at-least-32-characters-long")

	svc := NewAuthService(db)

	mock.ExpectQuery(regexp.QuoteMeta(`SELECT * FROM "users" WHERE username = $1 ORDER BY "users"."id" LIMIT $2`)).
		WithArgs("nonexistent", 1).
		WillReturnError(gorm.ErrRecordNotFound)

	accessToken, refreshToken, user, err := svc.Login(context.Background(), "nonexistent", "password123")

	assert.Error(t, err)
	assert.Equal(t, "invalid username or password", err.Error())
	assert.Empty(t, accessToken)
	assert.Empty(t, refreshToken)
	assert.Nil(t, user)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestLogin_InvalidPassword(t *testing.T) {
	db, mock := setupAuthTestDB(t)

	origEnv := os.Getenv("JWT_SECRET")
	defer os.Setenv("JWT_SECRET", origEnv)
	os.Setenv("JWT_SECRET", "test-secret-at-least-32-characters-long")

	svc := NewAuthService(db)

	correctPassword := "correct123"
	hash, _ := bcrypt.GenerateFromPassword([]byte(correctPassword), bcrypt.DefaultCost)

	rows := sqlmock.NewRows([]string{"id", "username", "password_hash", "role", "created_at", "updated_at"}).
		AddRow("user-123", "testuser", string(hash), "admin", time.Now(), time.Now())

	mock.ExpectQuery(regexp.QuoteMeta(`SELECT * FROM "users" WHERE username = $1 ORDER BY "users"."id" LIMIT $2`)).
		WithArgs("testuser", 1).
		WillReturnRows(rows)

	accessToken, refreshToken, user, err := svc.Login(context.Background(), "testuser", "wrongpassword")

	assert.Error(t, err)
	assert.Equal(t, "invalid username or password", err.Error())
	assert.Empty(t, accessToken)
	assert.Empty(t, refreshToken)
	assert.Nil(t, user)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestRefreshTokens_Success(t *testing.T) {
	db, mock := setupAuthTestDB(t)

	origEnv := os.Getenv("JWT_SECRET")
	defer os.Setenv("JWT_SECRET", origEnv)
	os.Setenv("JWT_SECRET", "test-secret-at-least-32-characters-long")

	svc := NewAuthService(db)

	userID := "user-123"
	refreshToken, err := svc.signRefreshToken(userID)
	assert.NoError(t, err)

	rows := sqlmock.NewRows([]string{"id", "username", "password_hash", "role", "created_at", "updated_at"}).
		AddRow(userID, "testuser", "hash", "admin", time.Now(), time.Now())

	mock.ExpectQuery(regexp.QuoteMeta(`SELECT * FROM "users" WHERE id = $1 ORDER BY "users"."id" LIMIT $2`)).
		WithArgs(userID, 1).
		WillReturnRows(rows)

	newAccess, newRefresh, err := svc.RefreshTokens(context.Background(), refreshToken)

	assert.NoError(t, err)
	assert.NotEmpty(t, newAccess)
	assert.NotEmpty(t, newRefresh)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestRefreshTokens_InvalidToken(t *testing.T) {
	db, _ := setupAuthTestDB(t)

	origEnv := os.Getenv("JWT_SECRET")
	defer os.Setenv("JWT_SECRET", origEnv)
	os.Setenv("JWT_SECRET", "test-secret-at-least-32-characters-long")

	svc := NewAuthService(db)

	newAccess, newRefresh, err := svc.RefreshTokens(context.Background(), "invalid-token")

	assert.Error(t, err)
	assert.Equal(t, "invalid refresh token", err.Error())
	assert.Empty(t, newAccess)
	assert.Empty(t, newRefresh)
}

func TestRefreshTokens_UserNotFound(t *testing.T) {
	db, mock := setupAuthTestDB(t)

	origEnv := os.Getenv("JWT_SECRET")
	defer os.Setenv("JWT_SECRET", origEnv)
	os.Setenv("JWT_SECRET", "test-secret-at-least-32-characters-long")

	svc := NewAuthService(db)

	userID := "nonexistent-user"
	refreshToken, err := svc.signRefreshToken(userID)
	assert.NoError(t, err)

	mock.ExpectQuery(regexp.QuoteMeta(`SELECT * FROM "users" WHERE id = $1 ORDER BY "users"."id" LIMIT $2`)).
		WithArgs(userID, 1).
		WillReturnError(gorm.ErrRecordNotFound)

	newAccess, newRefresh, err := svc.RefreshTokens(context.Background(), refreshToken)

	assert.Error(t, err)
	assert.Equal(t, "user not found", err.Error())
	assert.Empty(t, newAccess)
	assert.Empty(t, newRefresh)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestParseAccessToken_Success(t *testing.T) {
	db, _ := setupAuthTestDB(t)

	origEnv := os.Getenv("JWT_SECRET")
	defer os.Setenv("JWT_SECRET", origEnv)
	os.Setenv("JWT_SECRET", "test-secret-at-least-32-characters-long")

	svc := NewAuthService(db)

	userID := "user-123"
	role := "admin"
	accessToken, err := svc.signAccessToken(userID, role)
	assert.NoError(t, err)

	parsedUserID, parsedRole, err := svc.ParseAccessToken(accessToken)

	assert.NoError(t, err)
	assert.Equal(t, userID, parsedUserID)
	assert.Equal(t, role, parsedRole)
}

func TestParseAccessToken_InvalidToken(t *testing.T) {
	db, _ := setupAuthTestDB(t)

	origEnv := os.Getenv("JWT_SECRET")
	defer os.Setenv("JWT_SECRET", origEnv)
	os.Setenv("JWT_SECRET", "test-secret-at-least-32-characters-long")

	svc := NewAuthService(db)

	parsedUserID, parsedRole, err := svc.ParseAccessToken("invalid-token")

	assert.Error(t, err)
	assert.Empty(t, parsedUserID)
	assert.Empty(t, parsedRole)
}

func TestParseAccessToken_WrongSigningMethod(t *testing.T) {
	db, _ := setupAuthTestDB(t)

	origEnv := os.Getenv("JWT_SECRET")
	defer os.Setenv("JWT_SECRET", origEnv)
	os.Setenv("JWT_SECRET", "test-secret-at-least-32-characters-long")

	svc := NewAuthService(db)

	parsedUserID, parsedRole, err := svc.ParseAccessToken("eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0.eyJ1c2VyX2lkIjoiMTIzIiwicm9sZSI6ImFkbWluIn0.")

	assert.Error(t, err)
	assert.Contains(t, err.Error(), "unexpected signing method")
	assert.Empty(t, parsedUserID)
	assert.Empty(t, parsedRole)
}

func TestAccessToken_Expiration(t *testing.T) {
	db, _ := setupAuthTestDB(t)

	origEnv := os.Getenv("JWT_SECRET")
	defer os.Setenv("JWT_SECRET", origEnv)
	os.Setenv("JWT_SECRET", "test-secret-at-least-32-characters-long")

	svc := NewAuthService(db)

	userID := "user-123"
	accessToken, err := svc.signAccessToken(userID, "admin")
	assert.NoError(t, err)

	time.Sleep(1 * time.Second)

	parsedUserID, _, err := svc.ParseAccessToken(accessToken)
	assert.NoError(t, err)
	assert.Equal(t, userID, parsedUserID)
}
