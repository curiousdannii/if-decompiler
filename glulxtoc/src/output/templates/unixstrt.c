/* unixstrt.c: Unix-specific code for Glulxe.
    Designed by Andrew Plotkin <erkyrath@eblong.com>
    http://eblong.com/zarf/glulx/index.html
*/

#include <stdlib.h>
#include <string.h>
#include "glk.h"
#include "gi_blorb.h"
#include "glulxe.h"
#include "unixstrt.h"
#include "glkstart.h" /* This comes with the Glk library. */
#include "glulxtoc.h"

/* With glulxtoc the only argument is the number of undo states.
*/
glkunix_argumentlist_t glkunix_arguments[] = {
  { "--undo", glkunix_arg_ValueFollows, "Number of undo states to store." },
  { NULL, glkunix_arg_End, NULL }
};

int glkunix_startup_code(glkunix_startup_t *data)
{
  /* It turns out to be more convenient if we return TRUE from here, even 
     when an error occurs, and display an error in glk_main(). */
  int ix;
  unsigned char buf[12];
  int res;

  /* Parse out the arguments. They've already been checked for validity,
     and the library-specific ones stripped out.
     As usual for Unix, the zeroth argument is the executable name. */
  for (ix=1; ix<data->argc; ix++) {
    if (!strcmp(data->argv[ix], "--undo")) {
      ix++;
      if (ix<data->argc) {
        int val = atoi(data->argv[ix]);
        if (val <= 0) {
          init_err = "--undo must be a number.";
          return TRUE;
        }
        max_undo_level = val;
      }
      continue;
    }
  }

  gamefile = glk_stream_open_memory(GLULX_IMAGE, GLULX_IMAGE_LENGTH, filemode_Read, 1);
  if (!gamefile) {
    init_err = "The game file could not be opened.";
    return TRUE;
  }

  /* Now we have to check to see if it's a Blorb file. */

  glk_stream_set_position(gamefile, 0, seekmode_Start);
  res = glk_get_buffer_stream(gamefile, (char *)buf, 12);
  if (!res) {
    init_err = "The data in this stand-alone game is too short to read.";
    return TRUE;
  }

  if (buf[0] == 'G' && buf[1] == 'l' && buf[2] == 'u' && buf[3] == 'l') {
    /* Load game directly from file. */
    locate_gamefile(FALSE);
    return TRUE;
  }
  else if (buf[0] == 'F' && buf[1] == 'O' && buf[2] == 'R' && buf[3] == 'M'
    && buf[8] == 'I' && buf[9] == 'F' && buf[10] == 'R' && buf[11] == 'S') {
    /* Load game from a chunk in the Blorb file. */
    locate_gamefile(TRUE);
    return TRUE;
  }
  else {
    init_err = "This is neither a Glulx game file nor a Blorb file "
      "which contains one.";
    return TRUE;
  }
}