#define PY_SSIZE_T_CLEAN
#include <Python.h>

long inner_fibunacci(long n) {
    if (n <= 1) {
        return 1;
    }

    return inner_fibunacci(n-1) + inner_fibunacci(n-2);
}


static PyObject * fibunacci_c_fibunacci(PyObject *self, PyObject *args)
{
    const long n;
    if (!PyArg_ParseTuple(args, "i", &n)) {
        return NULL;
    }

    return PyLong_FromLong(inner_fibunacci(n));
}

static PyMethodDef fibunacci_cMethods[] = {

    {"fibunacci",  fibunacci_c_fibunacci, METH_VARARGS,
     "Fibunacci"},

    {NULL, NULL, 0, NULL}
};


static struct PyModuleDef fibunacci_c = {
    PyModuleDef_HEAD_INIT,
    "fibunacci_c",
    NULL,
    -1,
    fibunacci_cMethods
};

PyMODINIT_FUNC
PyInit_fibunacci_c(void)
{
    return PyModule_Create(&fibunacci_c);
}


int
main(int argc, char *argv[])
{
    wchar_t *program = Py_DecodeLocale(argv[0], NULL);
    if (program == NULL) {
        fprintf(stderr, "Fatal error: cannot decode argv[0]\n");
        exit(1);
    }


    if (PyImport_AppendInittab("fibunacci_c", PyInit_fibunacci_c) == -1) {
        fprintf(stderr, "Error: could not extend in-built modules table\n");
        exit(1);
    }


    Py_SetProgramName(program);


    Py_Initialize();



    if (!PyImport_ImportModule("fibunacci_c")) {
        PyErr_Print();
        fprintf(stderr, "Error: could not import module 'fibunacci_c'\n");
    }



    PyMem_RawFree(program);
    return 0;
}
