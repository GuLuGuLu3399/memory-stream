//go:build integration

package services

import (
	"testing"

	"github.com/DATA-DOG/go-sqlmock"
	"github.com/stretchr/testify/assert"
	"gorm.io/driver/postgres"
	"gorm.io/gorm"
)

func setupIntegrationTestDB(t *testing.T) (*gorm.DB, sqlmock.Sqlmock) {
	sqlDB, mock, err := sqlmock.New()
	if err != nil {
		t.Fatalf("failed to create sqlmock: %v", err)
	}

	dialector := postgres.New(postgres.Config{
		Conn: sqlDB,
	})
	db, err := gorm.Open(dialector, &gorm.Config{})
	if err != nil {
		t.Fatalf("failed to open gorm db: %v", err)
	}

	return db, mock
}

func TestSearchIntegration_EmptyQuery(t *testing.T) {
	db, _ := setupIntegrationTestDB(t)
	defer db.Exec("")

	svc := NewSearchService(db)

	results, total, err := svc.SearchCards("", 20, 0)

	assert.Error(t, err)
	assert.Equal(t, 0, total)
	assert.Nil(t, results)
	assert.Contains(t, err.Error(), "search query cannot be empty")
}

func TestSearchIntegration_LimitClamping_DefaultValue(t *testing.T) {
	db, mock := setupIntegrationTestDB(t)
	defer db.Exec("")

	svc := NewSearchService(db)

	countRows := sqlmock.NewRows([]string{"count"}).AddRow(5)
	mock.ExpectQuery("SELECT count").
		WillReturnRows(countRows)

	searchRows := sqlmock.NewRows([]string{"id", "title", "excerpt", "rank"}).
		AddRow("card-1", "Test", "Content", 0.5)
	mock.ExpectQuery("SELECT id.*rank").
		WithArgs(sqlmock.AnyArg(), sqlmock.AnyArg(), 20, 0).
		WillReturnRows(searchRows)

	results, total, err := svc.SearchCards("test", 0, 0)

	assert.NoError(t, err)
	assert.Equal(t, 5, total)
	assert.NotNil(t, results)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestSearchIntegration_LimitClamping_MaxClamp(t *testing.T) {
	db, mock := setupIntegrationTestDB(t)
	defer db.Exec("")

	svc := NewSearchService(db)

	countRows := sqlmock.NewRows([]string{"count"}).AddRow(5)
	mock.ExpectQuery("SELECT count").
		WillReturnRows(countRows)

	searchRows := sqlmock.NewRows([]string{"id", "title", "excerpt", "rank"}).
		AddRow("card-1", "Test", "Content", 0.5)
	mock.ExpectQuery("SELECT id.*rank").
		WithArgs(sqlmock.AnyArg(), sqlmock.AnyArg(), 100, 0).
		WillReturnRows(searchRows)

	results, total, err := svc.SearchCards("test", 500, 0)

	assert.NoError(t, err)
	assert.Equal(t, 5, total)
	assert.NotNil(t, results)
	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestSearchIntegration_OffsetClamping(t *testing.T) {
	db, mock := setupIntegrationTestDB(t)
	defer db.Exec("")

	svc := NewSearchService(db)

	countRows := sqlmock.NewRows([]string{"count"}).AddRow(5)
	mock.ExpectQuery("SELECT count").
		WillReturnRows(countRows)

	searchRows := sqlmock.NewRows([]string{"id", "title", "excerpt", "rank"}).
		AddRow("card-1", "Test", "Content", 0.5)
	mock.ExpectQuery("SELECT id.*rank").
		WithArgs(sqlmock.AnyArg(), sqlmock.AnyArg(), 20, 0).
		WillReturnRows(searchRows)

	results, total, err := svc.SearchCards("test", 20, -10)

	assert.NoError(t, err)
	assert.Equal(t, 5, total)
	assert.NotNil(t, results)
	assert.NoError(t, mock.ExpectationsWereMet())
}
