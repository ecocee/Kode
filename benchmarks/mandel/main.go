package main

import "fmt"

func main() {
	var count int64 = 0
	for py := 0; py < 300; py++ {
		for px := 0; px < 300; px++ {
			x0 := float64(float64(px)/300.0*3.5) - 2.5
			y0 := float64(float64(py)/300.0*2.0) - 1.0
			x, y := 0.0, 0.0
			i := 0
			for float64(x*x)+float64(y*y) <= 4.0 && i < 300 {
				xt := float64(float64(x*x)-float64(y*y)) + x0
				y = float64(float64(2.0*x)*y) + y0
				x = xt
				i++
			}
			count += int64(i)
		}
	}
	fmt.Println(count)
}
