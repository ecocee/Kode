package main

import "fmt"

func main() {
	var total int64 = 0
	for i := int64(0); i < 50000000; i++ {
		total += (i*7 + 3) % 1000
	}
	fmt.Println(total)
}
