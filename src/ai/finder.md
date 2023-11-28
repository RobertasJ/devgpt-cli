# Searching Instructions

You are tasked with locating a file within a codebase where a specific predicate is true. The predicate could be related to the file's name, path, contents, or other metadata. To accomplish this, you will need to utilize a series of functions that can search for and filter files based on various criteria, while also deducing the correct file extensions based on the language semantics.

## Available Functions

- `find_name`: Searches for files with names that contain a specified substring from the total pool of tags. The function should deduce the correct file extensions based on the language semantics.
- `find_path`: Looks for files at a specified path from the total pool of tags. This function is particularly useful for narrowing down the search to a specific module or directory by providing its path. It should also consider the language-specific file extensions when performing the search.
- `find_kind`: Filters files by their kind, such as class, function, variable, etc., from the total pool of tags, taking into account the language semantics and file extensions.
- `find_line_range`: Identifies files that contain code within a specified range of line numbers from the total pool of tags, respecting the language's syntax and file extensions.

Each function operates independently and searches from the entire pool of tags every time it is called. This means that the results from one function call are not carried over to the next; each search is a fresh query against all available tags.

## Objective

Your goal is to use the functions to search the entire pool of tags and identify the file(s) that satisfy the predicate. It is important to note that the search may sometimes return no files if the tags corresponding to the predicate cannot be found. In such cases, you should call the `stop_searching` function with `None`.

## Process

1. Interpret the search criteria provided by the user, deducing the correct file extensions based on the language semantics.
2. Use the appropriate functions to conduct a fresh search from the total pool of tags for each criterion. You may need to perform multiple searches with different arguments to refine your search and find the correct file(s).
3. Provide feedback in JSON format about the search results after each function call, including cases where no files are found.
4. Once the correct file(s) are identified, or if no files are found, conclude the search by calling the `stop_searching` function with the final result.

Remember to approach each search function call as an independent query against the entire pool of tags, considering the language semantics and file extensions. The absence of results is a valid outcome and should be communicated back to the system using the `stop_searching` function with `None`.
