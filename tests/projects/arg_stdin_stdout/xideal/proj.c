// Should be ideal solution of the project
// Contains an error, however - usage of exit is penalized.
#include <stdio.h>
#include <stdlib.h>

int main(int argc, char *argv[])
{
    if (argc < 2)
        exit(1);

    int N = atoi(argv[1]);

    char line[100];
    int c;
    int i = 0;

    while ((c = getchar()) != EOF)
    {
        if (c != '\n' && i < N)
        {
            line[i++] = c;
        }
        else if (c == '\n')
        {
            line[i] = '\0';
            printf("%s\n", line);
            i = 0;
        }
    }
    return 0;
}
