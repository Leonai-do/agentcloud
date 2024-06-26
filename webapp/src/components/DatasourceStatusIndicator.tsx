import ButtonSpinner from 'components/ButtonSpinner';
import ProgressBar from 'components/ProgressBar';
import { DatasourceStatus, datasourceStatusColors } from 'struct/datasource';

function DatasourceStatusIndicator({ datasource, processingOrEmbedding }) {
	const finished = datasource.recordCount?.total === null || ((datasource.recordCount?.success+datasource.recordCount?.failure) >= datasource.recordCount?.total);
	finished && (datasource.status = DatasourceStatus.READY);
	return (DatasourceStatus.EMBEDDING === datasource.status && !finished)
		? <ProgressBar total={datasource.recordCount?.total} success={datasource.recordCount?.success} failure={datasource.recordCount?.failure} />
		: <div className={`max-w-[300px] px-3 py-[2px] text-sm text-white text-center rounded-full capitalize ${(processingOrEmbedding && !finished) ? 'barberpole' : datasourceStatusColors[datasource.status]}`}>
			{datasource.status || 'Unknown'}{!finished && <ButtonSpinner size={14} className='ms-2 -me-1' />}
		</div>;
}

export default DatasourceStatusIndicator;
