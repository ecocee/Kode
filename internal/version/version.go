package version

import (
	"fmt"
	"runtime"
)

const (
	Version = "0.3.3"
)

func GetVersion() string {
	return fmt.Sprintf("Kode v%s\nBuilt with Go %s\nPlatform: %s/%s",
		Version,
		runtime.Version(),
		runtime.GOOS,
		runtime.GOARCH)
}
