/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under both the MIT license found in the
 * LICENSE-MIT file in the root directory of this source tree and the Apache
 * License, Version 2.0 found in the LICENSE-APACHE file in the root directory
 * of this source tree.
 */

#include <Python.h>
#include <stdlib.h>
#include <sys/time.h>

#include <optional>
#include <string>

namespace {

std::optional<int> MaybeGetExitCode(PyStatus* status, PyConfig* config) {
  if (PyStatus_IsExit(*status)) {
    return status->exitcode;
  }
  PyConfig_Clear(config);
  Py_ExitStatusException(*status);
  return std::nullopt;
}

} // namespace

extern struct _inittab _static_extension_info[];
PyMODINIT_FUNC PyInit__static_extension_utils();

int main(int argc, char* argv[]) {
  PyStatus status;
  PyConfig config;

  struct timeval tv;
  gettimeofday(&tv, nullptr);
  double current_time = (double)tv.tv_usec / 1000000.0 + tv.tv_sec;
  setenv("PAR_LAUNCH_TIMESTAMP", std::to_string(current_time).c_str(), true);

  PyConfig_InitPythonConfig(&config);

  status = PyConfig_SetBytesString(&config, &config.program_name, argv[0]);
  if (PyStatus_Exception(status)) {
    if (auto exit_code = MaybeGetExitCode(&status, &config)) {
      return *exit_code;
    }
  }

  // Read all configuration at once.
  status = PyConfig_Read(&config);
  if (PyStatus_Exception(status)) {
    if (auto exit_code = MaybeGetExitCode(&status, &config)) {
      return *exit_code;
    }
  }

  status = PyConfig_SetBytesArgv(&config, argc, argv);
  if (PyStatus_Exception(status)) {
    if (auto exit_code = MaybeGetExitCode(&status, &config)) {
      return *exit_code;
    }
  }

  // Check if we're using par_style="native", if so, modify sys.path to include
  // the executable-zipfile to it, and set a main module to run when invoked.
#ifdef NATIVE_PAR_STYLE
  // Append path to the executable itself to sys.path.
  status =
      PyWideStringList_Append(&config.module_search_paths, config.executable);
  if (PyStatus_Exception(status)) {
    if (auto exit_code = MaybeGetExitCode(&status, &config)) {
      return *exit_code;
    }
  }

  // Run entry-point module at startup.
  status =
      PyConfig_SetBytesString(&config, &config.run_module, "__run_npar_main__");
  if (PyStatus_Exception(status)) {
    if (auto exit_code = MaybeGetExitCode(&status, &config)) {
      return *exit_code;
    }
  }
#endif /* NATIVE_PAR_STYLE */

  // TODO (T129253406) We can do some code generation on build, we will have tho
  // full library name and the symbol name. FYI the currently we mangle symbol
  // names to avoid collision so PyInit_bye ->
  // PyInit_python_efficiency_experimental_linking_tests_bye
  // One issue with this is how we would resolve foo_bar.baz and foo.bar_baz in
  // both cases the name would be mangled to PyInit_foo_bar_baz...
  if (auto exit_code = PyImport_AppendInittab(
          "_static_extension_utils", PyInit__static_extension_utils);
      exit_code != 0) {
    PyErr_Print();
    fprintf(stderr, "Error: could not update inittab\n");
    return exit_code;
  }

  status = Py_InitializeFromConfig(&config);
  if (PyStatus_Exception(status)) {
    if (auto exit_code = MaybeGetExitCode(&status, &config)) {
      return *exit_code;
    }
  }

  // Call static_extension_finder._initialize()
  PyObject* pmodule = PyImport_ImportModule("static_extension_finder");
  if (pmodule == nullptr) {
    PyErr_Print();
    fprintf(
        stderr, "Error: could not import module 'static_extension_finder'\n");
    // return 1;
  } else {
    PyObject* pinitialize = PyObject_GetAttrString(pmodule, "_initialize");
    Py_DECREF(pmodule);
    if (pinitialize == nullptr || !PyCallable_Check(pinitialize)) {
      PyErr_Print();
      fprintf(
          stderr,
          "Error: could not find '_initialize' in module 'static_extension_finder'\n");
      // return 1;
    }
    PyObject* retvalue = PyObject_CallObject(pinitialize, nullptr);
    Py_DECREF(pinitialize);
    if (retvalue == nullptr) {
      PyErr_Print();
      fprintf(
          stderr,
          "Error: could not call 'static_extension_finder._initialize()'\n");
      // return 1;
    }
    Py_DECREF(retvalue);
  }
  PyConfig_Clear(&config);
  return Py_RunMain();
}
