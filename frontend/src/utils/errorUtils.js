export const getErrorMessage = (error) => {
    if (!error) return 'An unknown error occurred';
    if (error instanceof Error) return error.message;

    if (error.response?.data?.message) return error.response.data.message;
    if (error.response?.data) return JSON.stringify(error.response.data);
    if (error.message) return error.message;
    return String(error);
}