import { useState, useCallback, useEffect } from 'react';
import {
  Card,
  CardHeader,
  CardPreview,
  Button,
  Input,
  Dropdown,
  Option,
  Table,
  TableHeader,
  TableRow,
  TableHeaderCell,
  TableBody,
  TableCell,
  Dialog,
  DialogSurface,
  DialogTitle,
  DialogBody,
  DialogActions,
  DialogContent,
  TabList,
  Tab,
  Badge,
  Text,
  makeStyles,
  tokens,
  Spinner,
} from '@fluentui/react-components';
import {
  Search24Regular,
  Filter24Regular,
  ChevronLeft24Regular,
  ChevronRight24Regular,
} from '@fluentui/react-icons';
import { useI18n } from '../i18n';

interface LogItem {
  id: string;
  timestamp: number;
  protocol: string;
  model: string;
  provider_id?: string;
  stream: boolean;
  latency_ms: number;
  status_code: number;
  prompt_tokens: number;
  completion_tokens: number;
  cost: number;
  error_type?: string;
}

const useStyles = makeStyles({
  container: {
    display: 'flex',
    flexDirection: 'column',
    gap: tokens.spacingVerticalL,
    padding: tokens.spacingHorizontalL,
  },
  toolbar: {
    display: 'flex',
    gap: tokens.spacingHorizontalM,
    alignItems: 'center',
    flexWrap: 'wrap',
  },
  searchInput: {
    minWidth: '240px',
  },
  table: {
    minWidth: '100%',
  },
  pagination: {
    display: 'flex',
    justifyContent: 'flex-end',
    alignItems: 'center',
    gap: tokens.spacingHorizontalM,
    marginTop: tokens.spacingVerticalM,
  },
  successBadge: {
    backgroundColor: tokens.colorPaletteGreenBackground1,
    color: tokens.colorPaletteGreenForeground1,
  },
  errorBadge: {
    backgroundColor: tokens.colorPaletteRedBackground1,
    color: tokens.colorPaletteRedForeground1,
  },
  codeBlock: {
    backgroundColor: tokens.colorNeutralBackground2,
    padding: tokens.spacingHorizontalM,
    borderRadius: tokens.borderRadiusMedium,
    fontFamily: 'monospace',
    fontSize: '12px',
    maxHeight: '400px',
    overflow: 'auto',
    whiteSpace: 'pre-wrap',
    wordBreak: 'break-word',
  },
});

export default function LogsPage() {
  const { t } = useI18n();
  const styles = useStyles();

  const [logs, setLogs] = useState<LogItem[]>([]);
  const [loading, setLoading] = useState(false);
  const [search, setSearch] = useState('');
  const [statusFilter, setStatusFilter] = useState<string>('all');
  const [offset, setOffset] = useState(0);
  const [selectedLog, setSelectedLog] = useState<LogItem | null>(null);
  const [detailOpen, setDetailOpen] = useState(false);
  const [detailTab, setDetailTab] = useState('request');
  const [detailBody, setDetailBody] = useState({ request: '', response: '' });

  const limit = 20;

  const fetchLogs = useCallback(async () => {
    setLoading(true);
    try {
      const result = await (window as any).__TAURI_INVOKE__('query_logs', {
        params: {
          search: search || null,
          status: statusFilter === 'all' ? null : statusFilter,
          limit,
          offset,
        },
      });
      setLogs(result || []);
    } catch (e) {
      console.error('Failed to fetch logs', e);
    } finally {
      setLoading(false);
    }
  }, [search, statusFilter, offset]);

  useEffect(() => {
    fetchLogs();
  }, [fetchLogs]);

  const handleOpenDetail = async (log: LogItem) => {
    setSelectedLog(log);
    setDetailOpen(true);
    setDetailBody({ request: '', response: '' });
    try {
      const detail = await (window as any).__TAURI_INVOKE__('get_log_detail', {
        logId: log.id,
      });
      if (detail) {
        setDetailBody({
          request: detail.request_body,
          response: detail.response_body,
        });
      }
    } catch (e) {
      console.error('Failed to fetch log detail', e);
    }
  };

  const formatTime = (ts: number) => {
    return new Date(ts * 1000).toLocaleString();
  };

  return (
    <div className={styles.container}>
      <Card>
        <CardHeader
          header={<Text weight="semibold" size={500}>{t.logs.title}</Text>}
        />
        <CardPreview>
          <div style={{ padding: '16px' }}>
            <div className={styles.toolbar}>
              <Input
                className={styles.searchInput}
                placeholder={t.logs.search}
                value={search}
                onChange={(_, data) => setSearch(data.value)}
                contentBefore={<Search24Regular />}
                onKeyDown={(e) => { if (e.key === 'Enter') { setOffset(0); fetchLogs(); }}}
              />
              <Dropdown
                placeholder={t.logs.filter}
                value={statusFilter === 'all' ? t.logs.all : statusFilter === 'success' ? t.logs.success : t.logs.error}
                onOptionSelect={(_, data) => {
                  setStatusFilter((data.optionValue as string) || 'all');
                  setOffset(0);
                }}
              >
                <Option value="all">{t.logs.all}</Option>
                <Option value="success">{t.logs.success}</Option>
                <Option value="error">{t.logs.error}</Option>
              </Dropdown>
              <Button appearance="primary" onClick={() => { setOffset(0); fetchLogs(); }}>
                <Filter24Regular /> {t.logs.filter}
              </Button>
            </div>

            {loading ? (
              <Spinner style={{ margin: '40px auto', display: 'block' }} />
            ) : (
              <>
                <Table className={styles.table}>
                  <TableHeader>
                    <TableRow>
                      <TableHeaderCell>{t.logs.time}</TableHeaderCell>
                      <TableHeaderCell>{t.logs.model}</TableHeaderCell>
                      <TableHeaderCell>{t.logs.protocol}</TableHeaderCell>
                      <TableHeaderCell>{t.logs.method}</TableHeaderCell>
                      <TableHeaderCell>{t.logs.status}</TableHeaderCell>
                      <TableHeaderCell>{t.logs.latency}</TableHeaderCell>
                      <TableHeaderCell>{t.logs.tokens}</TableHeaderCell>
                      <TableHeaderCell>{t.logs.cost}</TableHeaderCell>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {logs.map((log) => (
                      <TableRow
                        key={log.id}
                        onClick={() => handleOpenDetail(log)}
                        style={{ cursor: 'pointer' }}
                      >
                        <TableCell>{formatTime(log.timestamp)}</TableCell>
                        <TableCell>{log.model}</TableCell>
                        <TableCell>{log.protocol}</TableCell>
                        <TableCell>{log.stream ? t.logs.streaming : t.logs.normal}</TableCell>
                        <TableCell>
                          {log.error_type || log.status_code >= 400 ? (
                            <Badge className={styles.errorBadge}>{t.logs.error}</Badge>
                          ) : (
                            <Badge className={styles.successBadge}>{t.logs.success}</Badge>
                          )}
                        </TableCell>
                        <TableCell>{log.latency_ms}ms</TableCell>
                        <TableCell>{log.prompt_tokens} → {log.completion_tokens}</TableCell>
                        <TableCell>${log.cost.toFixed(4)}</TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>

                <div className={styles.pagination}>
                  <Button
                    icon={<ChevronLeft24Regular />}
                    disabled={offset === 0}
                    onClick={() => setOffset(Math.max(0, offset - limit))}
                  />
                  <Text>{offset + 1} - {offset + logs.length}</Text>
                  <Button
                    icon={<ChevronRight24Regular />}
                    disabled={logs.length < limit}
                    onClick={() => setOffset(offset + limit)}
                  />
                </div>
              </>
            )}
          </div>
        </CardPreview>
      </Card>

      <Dialog open={detailOpen} onOpenChange={(_, data) => setDetailOpen(data.open)}>
        <DialogSurface>
          <DialogBody>
            <DialogTitle>{t.logs.detail}</DialogTitle>
            <DialogContent>
              {selectedLog && (
                <div style={{ marginBottom: '12px' }}>
                  <Text>
                    {formatTime(selectedLog.timestamp)} · {selectedLog.model} · {selectedLog.protocol} · {selectedLog.latency_ms}ms
                  </Text>
                </div>
              )}
              <TabList
                selectedValue={detailTab}
                onTabSelect={(_, data) => setDetailTab((data.value as string) || 'request')}
              >
                <Tab value="request">{t.logs.request}</Tab>
                <Tab value="response">{t.logs.response}</Tab>
              </TabList>
              <div style={{ marginTop: '12px' }}>
                <pre className={styles.codeBlock}>
                  {detailTab === 'request'
                    ? detailBody.request || '...'
                    : detailBody.response || '...'}
                </pre>
              </div>
            </DialogContent>
            <DialogActions>
              <Button appearance="secondary" onClick={() => setDetailOpen(false)}>
                {t.common.close}
              </Button>
            </DialogActions>
          </DialogBody>
        </DialogSurface>
      </Dialog>
    </div>
  );
}
