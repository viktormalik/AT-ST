// Prints first N characters of each word (will work for single-word lines only)
// Moreover contains errors - global variables and include of <string.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

char word[101];
char prefix[100];

int main(int argc, char *argv[])
{
    if (argc != 2) {
      fprintf(stderr, "incorrect number of arguments\n");
      return 1;
    }

    int N = atoi(argv[1]);

    while (scanf("%100s", word) == 1)
    {
        strncpy(prefix, word, N);
        prefix[N] = '\0';
        printf("%s\n", prefix);
    }

    return 0;
}
