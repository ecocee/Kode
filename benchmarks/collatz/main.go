package main

import "fmt"

func main() {
	var best, bestSteps int64
	for n := int64(1); n < 1500000; n++ {
		x, steps := n, int64(0)
		for x != 1 {
			if x%2 == 0 {
				x = x / 2
			} else {
				x = 3*x + 1
			}
			steps++
		}
		if steps > bestSteps {
			bestSteps = steps
			best = n
		}
	}
	fmt.Println(best)
}
