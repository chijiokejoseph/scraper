package main

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"

)

const URL = "https://web.archive.org/web/20200518073855/https://www.empireonline.com/movies/features/best-movies-2/"

var moviesChan = make(chan []Movie)

func DataPath() (string, error) {
	// get working directory
	dir, err := os.Getwd()
	if err != nil {
		return "", err
	}
	// construct data path
	dir = filepath.Join(dir, "data")
	// create data path directory
	err = os.MkdirAll(dir, os.ModeDir)
	if err != nil {
		return "", err
	}
	return dir, nil
}

func main() {
	// get start time
	start := time.Now()

	// launch go routine to scrape url
	go func() {
		movies, err := Scrape(URL, NewScrapeOpts().WithSaveWeb(false).WithSort(Ascending))
		if err != nil {
			return
		}
		moviesChan <- movies
	}()

	// setup banners for display
	spaces := 70
	banner := strings.Repeat("=", spaces)

	// display url
	fmt.Printf("URL = %s\n", URL)
	fmt.Println()

	// set up controls for display
	wait := 60000 // ms
	num := 25
	limit := 10
	i := 0
	easeTime := 0

	// run a for loop to make intermittent displays
	// catch the result of the url if complete
	// catch when the program timesout
	for {
		select {
		// if the result of url scrape is ready
		case movies := <-moviesChan:
			elapsed := time.Since(start)

			// show that url parsing is complete
			progress := strings.Repeat("=", num)
			fmt.Printf("\rParsing URL ... [%-*s] | %.2f%%", num, progress, 100.0)

			// display first 10 movies in the rating
			fmt.Println()
			fmt.Println()
			fmt.Println(banner)
			for _, movie := range movies[:limit] {
				fmt.Printf("%d. %s\n", movie.Num, movie.Title)
			}
			fmt.Println(banner)

			// display elapsed time
			fmt.Println()
			fmt.Printf("Elapsed Time: %d ms\n", elapsed.Milliseconds())
			fmt.Println()
			return
		// intermittent displays after interval while the url result is not ready
		case <-time.After(time.Duration(easeTime) * time.Millisecond):
			if i == 0 {
				i = 1
			}
			i++
			easeTime = wait * ((2 * i) - 1) / (num * num)
			progress := strings.Repeat("=", i)
			percent := float64(i) * 100.0 / float64(num)
			fmt.Printf("\rParsing URL ... [%-*s] | %.2f%%", num, progress, percent)
		// url not ready and program has timed out
		case <-time.After(time.Duration(wait) * time.Millisecond):
			fmt.Println()
			fmt.Printf("❌ Failed to load resource in %d ms\n", wait)
			fmt.Println()
			return
		}
	}
}
