package services

import (
	"context"
	"errors"
	"fmt"
	"os"
	"time"

	"github.com/GuLuGuLu3399/memory-stream-server/internal/models"
	"github.com/GuLuGuLu3399/memory-stream-server/internal/pkg/logger"
	"github.com/golang-jwt/jwt/v5"
	"golang.org/x/crypto/bcrypt"
	"gorm.io/gorm"
)

type AuthService struct {
	db     *gorm.DB
	jwtKey []byte
}

func NewAuthService(db *gorm.DB) *AuthService {
	secret := os.Getenv("JWT_SECRET")
	if secret == "" {
		if os.Getenv("GO_ENV") == "production" {
			panic("FATAL: JWT_SECRET environment variable must be set in production. Refusing to start with insecure default.")
		}
		logger.Log.Warnf("⚠️  WARNING: Using default JWT secret — NOT for production! Set JWT_SECRET environment variable.")
		secret = "memory-stream-dev-secret-change-in-production"
	}
	if len(secret) < 32 {
		panic("FATAL: JWT_SECRET must be at least 32 characters for HS256 security")
	}
	return &AuthService{db: db, jwtKey: []byte(secret)}
}

type AccessTokenClaims struct {
	jwt.RegisteredClaims
	UserID string `json:"user_id"`
	Role   string `json:"role"`
}

type RefreshTokenClaims struct {
	jwt.RegisteredClaims
	UserID string `json:"user_id"`
	JTI    string `json:"jti"`
}

func (s *AuthService) Register(ctx context.Context, username, password string) (*models.User, error) {
	if username == "" || password == "" {
		return nil, errors.New("username and password are required")
	}
	if len(password) < 6 {
		return nil, errors.New("password must be at least 6 characters")
	}

	var existing models.User
	if err := s.db.WithContext(ctx).Where("username = ?", username).First(&existing).Error; err == nil {
		return nil, errors.New("username already exists")
	}

	hash, err := bcrypt.GenerateFromPassword([]byte(password), bcrypt.DefaultCost)
	if err != nil {
		return nil, err
	}

	user := &models.User{
		Username:     username,
		PasswordHash: string(hash),
		Role:         "guest",
	}
	if err := s.db.WithContext(ctx).Create(user).Error; err != nil {
		return nil, err
	}
	return user, nil
}

// GenesisAdmin 创世接口：一次性创建 admin + guest 账号。
// 如果数据库中已存在 admin 用户，返回 ErrGenesisSealed。
// 使用可序列化事务防止多实例并发创建多个 admin。
func (s *AuthService) GenesisAdmin(ctx context.Context, username, password string) (*models.User, error) {
	var admin *models.User
	err := s.db.WithContext(ctx).Transaction(func(tx *gorm.DB) error {
		// 在事务内加锁检查是否已有 admin，防止并发竞态
		var adminCount int64
		if err := tx.Model(&models.User{}).Where("role = ?", "admin").Count(&adminCount).Error; err != nil {
			return fmt.Errorf("failed to count admin users: %w", err)
		}
		if adminCount > 0 {
			return ErrGenesisSealed
		}

		if username == "" || password == "" {
			return errors.New("username and password are required")
		}
		if len(password) < 6 {
			return errors.New("password must be at least 6 characters")
		}

		// 创建 admin 账号
		hash, err := bcrypt.GenerateFromPassword([]byte(password), bcrypt.DefaultCost)
		if err != nil {
			return fmt.Errorf("failed to hash password: %w", err)
		}

		admin = &models.User{
			Username:     username,
			PasswordHash: string(hash),
			Role:         "admin",
		}
		if err := tx.Create(admin).Error; err != nil {
			return fmt.Errorf("failed to create admin user: %w", err)
		}
		// 自动创建默认 guest 账号（用于前端无感登录）
		guestHash, err := bcrypt.GenerateFromPassword([]byte("guest123"), bcrypt.DefaultCost)
		if err != nil {
			return fmt.Errorf("failed to hash guest password: %w", err)
		}
		
		guest := &models.User{
			Username:     "guest",
			PasswordHash: string(guestHash),
			Role:         "guest",
		}
		if err := tx.Create(guest).Error; err != nil {
			return fmt.Errorf("failed to create guest user: %w", err)
		}
		
		return nil
	})
	if err != nil {
		return nil, err
	}

	return admin, nil
}

// IsGenesisSealed 检查创世大门是否已关闭（是否已有 admin）
func (s *AuthService) IsGenesisSealed(ctx context.Context) bool {
	var adminCount int64
	s.db.WithContext(ctx).Model(&models.User{}).Where("role = ?", "admin").Count(&adminCount)
	return adminCount > 0
}

var ErrGenesisSealed = errors.New("genesis already completed: admin account exists")

func (s *AuthService) Login(ctx context.Context, username, password string) (string, string, *models.User, error) {
	var user models.User
	if err := s.db.WithContext(ctx).Where("username = ?", username).First(&user).Error; err != nil {
		return "", "", nil, errors.New("invalid username or password")
	}

	if err := bcrypt.CompareHashAndPassword([]byte(user.PasswordHash), []byte(password)); err != nil {
		return "", "", nil, errors.New("invalid username or password")
	}

	accessToken, err := s.signAccessToken(user.ID, user.Role)
	if err != nil {
		return "", "", nil, err
	}

	refreshToken, err := s.signRefreshToken(user.ID)
	if err != nil {
		return "", "", nil, err
	}

	return accessToken, refreshToken, &user, nil
}

func (s *AuthService) RefreshTokens(ctx context.Context, refreshTokenStr string) (string, string, error) {
	claims, err := s.parseRefreshToken(refreshTokenStr)
	if err != nil {
		return "", "", errors.New("invalid refresh token")
	}

	var user models.User
	if err := s.db.WithContext(ctx).Where("id = ?", claims.UserID).First(&user).Error; err != nil {
		return "", "", errors.New("user not found")
	}

	newAccess, err := s.signAccessToken(user.ID, user.Role)
	if err != nil {
		return "", "", err
	}

	newRefresh, err := s.signRefreshToken(user.ID)
	if err != nil {
		return "", "", err
	}

	return newAccess, newRefresh, nil
}

func (s *AuthService) ParseAccessToken(tokenStr string) (string, string, error) {
	token, err := jwt.ParseWithClaims(tokenStr, &AccessTokenClaims{}, func(t *jwt.Token) (interface{}, error) {
		if _, ok := t.Method.(*jwt.SigningMethodHMAC); !ok {
			return nil, fmt.Errorf("unexpected signing method: %v", t.Header["alg"])
		}
		return s.jwtKey, nil
	})
	if err != nil {
		return "", "", err
	}
	claims, ok := token.Claims.(*AccessTokenClaims)
	if !ok || !token.Valid {
		return "", "", errors.New("invalid access token")
	}
	return claims.UserID, claims.Role, nil
}

func (s *AuthService) signAccessToken(userID, role string) (string, error) {
	claims := AccessTokenClaims{
		RegisteredClaims: jwt.RegisteredClaims{
			ExpiresAt: jwt.NewNumericDate(time.Now().Add(2 * time.Hour)),
			IssuedAt:  jwt.NewNumericDate(time.Now()),
			Issuer:    "memory-stream",
		},
		UserID: userID,
		Role:   role,
	}
	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	return token.SignedString(s.jwtKey)
}

func (s *AuthService) signRefreshToken(userID string) (string, error) {
	claims := RefreshTokenClaims{
		RegisteredClaims: jwt.RegisteredClaims{
			ExpiresAt: jwt.NewNumericDate(time.Now().Add(7 * 24 * time.Hour)),
			IssuedAt:  jwt.NewNumericDate(time.Now()),
			Issuer:    "memory-stream",
		},
		UserID: userID,
	}
	token := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	return token.SignedString(s.jwtKey)
}

func (s *AuthService) parseRefreshToken(tokenStr string) (*RefreshTokenClaims, error) {
	token, err := jwt.ParseWithClaims(tokenStr, &RefreshTokenClaims{}, func(t *jwt.Token) (interface{}, error) {
		if _, ok := t.Method.(*jwt.SigningMethodHMAC); !ok {
			return nil, fmt.Errorf("unexpected signing method: %v", t.Header["alg"])
		}
		return s.jwtKey, nil
	})
	if err != nil {
		return nil, err
	}
	claims, ok := token.Claims.(*RefreshTokenClaims)
	if !ok || !token.Valid {
		return nil, errors.New("invalid refresh token")
	}
	return claims, nil
}
