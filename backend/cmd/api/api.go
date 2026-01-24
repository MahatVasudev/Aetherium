package api

import (
	"context"
	"database/sql"
	"log"
	"net/http"
	"time"

	pdb "github.com/MahatVasudev/Aetherium/backend/proto/search"
	"github.com/go-chi/chi/v5"
	"github.com/go-chi/chi/v5/middleware"
	"github.com/go-chi/cors"
	"google.golang.org/grpc"
)

type APISERVER struct {
	Addr    string
	sqlDB   *sql.DB
	cacheDB *sql.DB
}

func NewAPISERVER(addr string, sqlDB *sql.DB, cacheDB *sql.DB) *APISERVER {
	return &APISERVER{
		Addr:    addr,
		sqlDB:   sqlDB,
		cacheDB: cacheDB,
	}
}

func (s *APISERVER) GRPC_LISTEN() {
	conn, _ := grpc.NewClient(s.Addr)

	defer conn.Close()

	client := pdb.NewSearchServiceClient(conn)

	ctx, cancel := context.WithTimeout(context.Background(), time.Second)
	defer cancel()
	resp, _ := client.Search(ctx, &pdb.SearchRequest{Query: "hello"})
	log.Println("Search results:", resp.Results)
}

func (newapiserver *APISERVER) HTTP_RUN() error {
	router := chi.NewRouter()

	router.Use(middleware.Logger)

	c := cors.Handler(
		cors.Options{
			AllowedOrigins:   []string{"http://localhost:5173"},
			AllowedMethods:   []string{"GET", "POST", "PUT", "DELETE", "OPTIONS"},
			AllowedHeaders:   []string{"Origin", "Content-Type", "Accept", "Authorization"},
			AllowCredentials: true,
			MaxAge:           300,
		},
	)

	router.Use(c)

	server := &http.Server{
		Addr:    newapiserver.Addr,
		Handler: router,
	}

	log.Printf("Connecting to Port %s....", newapiserver.Addr)

	return server.ListenAndServe()
}
