
package main

import "fmt"

func ack(m, n int) int {
	if m == 0 {
		return n + 1
	}
	if n == 0 {
		return ack(m-1, 1)
	}
	return ack(m-1, ack(m, n-1))
}

func main() {
	total := 0

	for i := 0; i < 100; i++ {
		total += ack(3, 8)
	}

	fmt.Println(total)
}
