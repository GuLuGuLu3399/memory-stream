package logger

import (
	"fmt"

	"go.uber.org/zap"
)

var Log *zap.SugaredLogger

func Init() error {
	l, err := zap.NewDevelopment()
	if err != nil {
		return fmt.Errorf("failed to initialize logger: %w", err)
	}
	Log = l.Sugar()
	return nil
}

func Sync() {
	if Log != nil {
		_ = Log.Sync()
	}
}
