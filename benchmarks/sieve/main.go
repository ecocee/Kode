package main

import "fmt"

func main() {
	n := 10000000
	f := make([]bool, 0)
	for i := 0; i <= n; i++ {
		f = append(f, true)
	}
	f[0] = false
	f[1] = false
	for i := 2; i*i <= n; i++ {
		if f[i] {
			for j := i * i; j <= n; j += i {
				f[j] = false
			}
		}
	}
	count := 0
	for k := 0; k <= n; k++ {
		if f[k] {
			count++
		}
	}
	fmt.Println(count)
}
