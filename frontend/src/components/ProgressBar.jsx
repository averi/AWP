const ProgressBar = ({ progress }) => (
  <div className="w-full bg-gray-200 rounded-full h-4">
    <div
      className="bg-green-500 h-4 transition-all duration-300 rounded-full"
      style={{ width: `${progress}%` }}
    />
  </div>
);

export default ProgressBar;