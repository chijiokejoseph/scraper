package main

import (
	"fmt"
	"os"
	"path/filepath"
	"slices"
	"strings"

	"github.com/gocolly/colly/v2"
)

type Movie struct {
	Num   int
	Title string
}

type Order int

const (
	Ascending Order = iota
	Descending
)

type ScrapeOpts struct {
	SaveWeb bool
	Sort    Order
}

func (opts *ScrapeOpts) WithSort(sort Order) *ScrapeOpts {
	opts.Sort = sort
	return opts
}

func (opts *ScrapeOpts) WithSaveWeb(saveWeb bool) *ScrapeOpts {
	opts.SaveWeb = saveWeb
	return opts
}

func NewScrapeOpts() *ScrapeOpts {
	return &ScrapeOpts{}
}

func Scrape(url string, opts *ScrapeOpts) ([]Movie, error) {
	var movies []Movie
	selectors := ".article-title-description .article-title-description__text .title"

	// get scrape settings
	sort, saveWeb := opts.Sort, opts.SaveWeb

	// get data path
	dir, err := DataPath()
	if err != nil {
		fmt.Println("Error getting Dir Path")
		return movies, err
	}

	// create new colly scraper client/collector
	coll := colly.NewCollector()

	// register and run callback on response from colly
	coll.OnResponse(func(r *colly.Response) {
		// handle bad status codes
		if r.StatusCode != 200 {
			fmt.Printf("Returned Status Code = %d", r.StatusCode)
			return
		}

		// if settings to save web page is false stop execution
		if !saveWeb {
			return
		}

		// get html
		htmlStr := string(r.Body)
		// construct file path
		filePath := filepath.Join(dir, "live.html")

		// open file in create/write mode
		file, err := os.OpenFile(filePath, os.O_CREATE+os.O_RDWR, os.ModePerm)
		if err != nil {
			fmt.Println("Failed to open file")
			fmt.Println(err)
			return
		}

		// handle file closing
		defer func() {
			err := file.Close()
			if err != nil {
				fmt.Printf("Error Closing File %s. Error %s", file.Name(), err)
			}
		}()

		// write to file
		_, err = file.WriteString(htmlStr)
		if err != nil {
			fmt.Printf("Error writing to file %s\n", file.Name())
		}
	})

	// register and run callback on retrieved html from selectors
	coll.OnHTML(selectors, func(h *colly.HTMLElement) {
		// get html content as string
		text := h.Text
		parts := strings.Split(text, ")")

		// if retrieved content is malformed return
		if len(parts) == 1 || len(parts) == 0 {
			return
		}

		// get movie num string and movie title string
		var partNum, partTitle string = parts[0], parts[1]

		// format movie num and title strings
		partNum = strings.Trim(partNum, " ")
		partTitle = strings.Trim(strings.TrimPrefix(partTitle, ")"), " ")

		// parse movie num string to a number
		var num int
		_, err = fmt.Sscanf(partNum, "%d", &num)
		if err != nil {
			fmt.Printf("Could not parse %s to number", partNum)
			return
		}

		// append movie details (Movie struct) to `movies`
		movie := Movie{Num: num, Title: partTitle}
		movies = append(movies, movie)
	})

	// attempt to visit url
	err = coll.Visit(url)
	// if visit fails return
	if err != nil {
		fmt.Printf("Encountered error %s", err)
		return movies, err
	}

	// sort movies slice based on movie rating numbers
	slices.SortFunc(movies, func(a, b Movie) int {
		switch sort {
		case Ascending:
			return a.Num - b.Num
		case Descending:
			return b.Num - a.Num
		default:
			return a.Num - b.Num
		}
	})

	// format movies printout
	var print string
	for _, movie := range movies {
		print += fmt.Sprintf("%d) %s\n", movie.Num, movie.Title)
	}
	// construct save path for movie ratings
	savePath := filepath.Join(dir, "movies.txt")

	// open file in create and write modes
	file, err := os.OpenFile(savePath, os.O_CREATE+os.O_RDWR, os.ModePerm)
	if err != nil {
		fmt.Printf("Error opening file '%s' with err: '%s'\n", file.Name(), err)
		return movies, err
	}

	// close file
	defer func() {
		err := file.Close()
		if err != nil {
			fmt.Printf("Error Closing File. Error: %s", err)
		}
	}()

	// write movies printout to file
	_, err = file.WriteString(print)
	if err != nil {
		fmt.Printf("Error writing to file %s\n", file.Name())
		return movies, err
	}

	return movies, nil
}
