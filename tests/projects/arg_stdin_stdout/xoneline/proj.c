// Handles only single line
// In addition contains a warning (unused argc)
#include <stdio.h>

int main(int argc, char *argv[])
{
    int N = atoi(argv[1]);

    char line[100];
    int c;
    int i = 0;

    while ((c = getchar()) != EOF && i < N)
        line[i++] = c;
    line[i] = '\0';

    printf("%s\n", line);
    return 0;
}
