package services

import (
	"testing"

	"github.com/DATA-DOG/go-sqlmock"
	"github.com/stretchr/testify/assert"
	"gorm.io/driver/postgres"
	"gorm.io/gorm"
)

func setupSearchTestDB(t *testing.T) (*gorm.DB, sqlmock.Sqlmock) {
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

func TestSearchCards_BasicSearch(t *testing.T) {
	db, mock := setupSearchTestDB(t)
	defer db.Exec("")

	svc := NewSearchService(db)

	countRows := sqlmock.NewRows([]string{"count"}).AddRow(2)
	mock.ExpectQuery("SELECT count").WithArgs("golang").WillReturnRows(countRows)

	searchRows := sqlmock.NewRows([]string{"id", "title", "excerpt", "rank"}).
		AddRow("uuid-1", "Golang Tutorial", "Learn Go programming", 0.9).
		AddRow("uuid-2", "Go Basics", "Introduction to Go", 0.7)
	mock.ExpectQuery("SELECT id.*rank").WithArgs("golang", "golang", 20, 0).WillReturnRows(searchRows)

	results, total, err := svc.SearchCards("golang", 20, 0)

	assert.NoError(t, err)
	assert.Equal(t, 2, total)
	assert.Len(t, results, 2)
	assert.Equal(t, "uuid-1", results[0].ID)
	assert.Equal(t, "Golang Tutorial", results[0].Title)

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestSearchCards_ChineseSearch(t *testing.T) {
	db, mock := setupSearchTestDB(t)
	defer db.Exec("")

	svc := NewSearchService(db)

	countRows := sqlmock.NewRows([]string{"count"}).AddRow(1)
	mock.ExpectQuery("SELECT count").WithArgs("知识图谱").WillReturnRows(countRows)

	searchRows := sqlmock.NewRows([]string{"id", "title", "excerpt", "rank"}).
		AddRow("uuid-4", "知识图谱", "这是一个知识图谱系统", 0.9)
	mock.ExpectQuery("SELECT id.*rank").WithArgs("知识图谱", "知识图谱", 20, 0).WillReturnRows(searchRows)

	results, total, err := svc.SearchCards("知识图谱", 20, 0)

	assert.NoError(t, err)
	assert.Equal(t, 1, total)
	assert.Len(t, results, 1)
	assert.Equal(t, "uuid-4", results[0].ID)
	assert.Equal(t, "知识图谱", results[0].Title)
	assert.Equal(t, "这是一个知识图谱系统", results[0].Excerpt)

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestSearchCards_EmptyQuery(t *testing.T) {
	db, _ := setupSearchTestDB(t)
	defer db.Exec("")

	svc := NewSearchService(db)

	results, total, err := svc.SearchCards("", 20, 0)

	assert.Error(t, err)
	assert.Equal(t, 0, total)
	assert.Nil(t, results)
	assert.Contains(t, err.Error(), "search query cannot be empty")
}

func TestSearchCards_Pagination(t *testing.T) {
	db, mock := setupSearchTestDB(t)
	defer db.Exec("")

	svc := NewSearchService(db)

	countRows := sqlmock.NewRows([]string{"count"}).AddRow(25)
	mock.ExpectQuery("SELECT count").WithArgs("test").WillReturnRows(countRows)

	searchRows := sqlmock.NewRows([]string{"id", "title", "excerpt", "rank"}).
		AddRow("uuid-11", "Result 11", "Content 11", 0.4).
		AddRow("uuid-12", "Result 12", "Content 12", 0.35)
	mock.ExpectQuery("SELECT id.*rank").WithArgs("test", "test", 10, 10).WillReturnRows(searchRows)

	results, total, err := svc.SearchCards("test", 10, 10)

	assert.NoError(t, err)
	assert.Equal(t, 25, total)
	assert.Len(t, results, 2)

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestSearchCards_NoResults(t *testing.T) {
	db, mock := setupSearchTestDB(t)
	defer db.Exec("")

	svc := NewSearchService(db)

	countRows := sqlmock.NewRows([]string{"count"}).AddRow(0)
	mock.ExpectQuery("SELECT count").WithArgs("nonexistent").WillReturnRows(countRows)

	searchRows := sqlmock.NewRows([]string{"id", "title", "excerpt", "rank"})
	mock.ExpectQuery("SELECT id.*rank").WithArgs("nonexistent", "nonexistent", 20, 0).WillReturnRows(searchRows)

	results, total, err := svc.SearchCards("nonexistent", 20, 0)

	assert.NoError(t, err)
	assert.Equal(t, 0, total)
	assert.Len(t, results, 0)

	assert.NoError(t, mock.ExpectationsWereMet())
}

func TestSearchCards_LimitCappedTo100(t *testing.T) {
	db, mock := setupSearchTestDB(t)
	defer db.Exec("")

	svc := NewSearchService(db)

	countRows := sqlmock.NewRows([]string{"count"}).AddRow(5)
	mock.ExpectQuery("SELECT count").WithArgs("test").WillReturnRows(countRows)

	searchRows := sqlmock.NewRows([]string{"id", "title", "excerpt", "rank"}).
		AddRow("uuid-1", "Test", "Content", 0.5)
	mock.ExpectQuery("SELECT id.*rank").WithArgs("test", "test", 100, 0).WillReturnRows(searchRows)

	results, total, err := svc.SearchCards("test", 200, 0)

	assert.NoError(t, err)
	assert.Equal(t, 5, total)
	assert.Len(t, results, 1)

	assert.NoError(t, mock.ExpectationsWereMet())
}
