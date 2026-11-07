// HTTP benchmark server, Go standard library only.
package main

import (
	"fmt"
	"net/http"
	"os"
	"strings"
)

func main() {
	mux := http.NewServeMux()
	mux.HandleFunc("/hello", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "text/plain")
		fmt.Fprint(w, "hello")
	})
	mux.HandleFunc("/json", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		fmt.Fprint(w, `{"message":"hello","ok":true,"n":1}`)
	})
	port := "8080"
	if len(os.Args) > 1 {
		port = strings.TrimPrefix(os.Args[1], ":")
	}
	srv := &http.Server{Addr: ":" + port, Handler: mux}
	fmt.Println("listening on", port)
	if err := srv.ListenAndServe(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}
