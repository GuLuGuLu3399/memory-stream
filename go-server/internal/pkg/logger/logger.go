package logger

import "go.uber.org/zap"

var Log *zap.SugaredLogger

func Init() {
	l, _ := zap.NewProduction()
	Log = l.Sugar()
}

func Sync() {
	if Log != nil {
		_ = Log.Sync()
	}
}
