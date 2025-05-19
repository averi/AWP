// Copyright: (c) 2025, Andrea Veri <andrea.veri@gmail.com>
// GNU General Public License v3.0+ (see COPYING or https://www.gnu.org/licenses/gpl-3.0.txt)

const ProgressBar = ({ progress }) => (
  <div className="w-full bg-gray-200 rounded-full h-4">
    <div
      className="bg-green-500 h-4 transition-all duration-300 rounded-full"
      style={{ width: `${progress}%` }}
    />
  </div>
);

export default ProgressBar;