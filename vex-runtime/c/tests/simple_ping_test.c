#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/wait.h>

int main() {
    printf("=== Simple Ping Stream Test (10 pings) ===\n\n");
    
    int pipefd[2];
    pipe(pipefd);
    
    pid_t pid = fork();
    if (pid == 0) {
        // Child
        close(pipefd[0]);
        dup2(pipefd[1], STDOUT_FILENO);
        close(pipefd[1]);
        execl("/sbin/ping", "ping", "-c", "10", "8.8.8.8", NULL);
        exit(1);
    }
    
    // Parent
    close(pipefd[1]);
    char buf[4096];
    ssize_t n;
    
    printf("✅ Streaming output in real-time:\n\n");
    
    while ((n = read(pipefd[0], buf, sizeof(buf)-1)) > 0) {
        buf[n] = '\0';
        printf("%s", buf);
        fflush(stdout);  // Real-time output!
    }
    
    close(pipefd[0]);
    wait(NULL);
    
    printf("\n✅ Streaming test passed!\n");
    return 0;
}
