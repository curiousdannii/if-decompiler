/* exec.c: Glulxe code for program execution. The main interpreter loop.
    Designed by Andrew Plotkin <erkyrath@eblong.com>
    http://eblong.com/zarf/glulx/index.html
*/

#include "glk.h"
#include "glulxe.h"
#include "opcodes.h"

#ifdef FLOAT_SUPPORT
#include <math.h>
#endif /* FLOAT_SUPPORT */

/* execute_loop():
   The main interpreter loop. This repeats until the program is done.
*/
void execute_loop()
{
  int done_executing = FALSE;
  int ix;
  glui32 opcode;
  operandlist_t *oplist;
  oparg_t inst[MAX_OPERANDS];
  glui32 value, addr, val0, val1;
  glsi32 vals0, vals1;
  glui32 *arglist;
  glui32 arglistfix[3];
#ifdef FLOAT_SUPPORT
  gfloat32 valf, valf1, valf2;
#endif /* FLOAT_SUPPORT */

}