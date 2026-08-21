import { HttpResponse } from '@angular/common/http';
import { ChangeDetectionStrategy, Component, OnInit, inject, signal } from '@angular/core';
import { ReactiveFormsModule } from '@angular/forms';
import { ActivatedRoute } from '@angular/router';

import { FontAwesomeModule } from '@fortawesome/angular-fontawesome';
import { TranslatePipe } from '@ngx-translate/core';
import { Observable, finalize, map } from 'rxjs';

import { DataUtils, FileLoadError } from 'app/core/util/data-util.service';
import { EventManager, EventWithContent } from 'app/core/util/event-manager.service';
import { ICvResume } from 'app/entities/cv-resume/cv-resume.model';
import { CvResumeService } from 'app/entities/cv-resume/service/cv-resume.service';
import { AlertError } from 'app/shared/alert/alert-error';
import { AlertErrorModel } from 'app/shared/alert/alert-error.model';
import { TranslateDirective } from 'app/shared/language';
import { ICvResumeVersion } from '../cv-resume-version.model';
import { CvResumeVersionService } from '../service/cv-resume-version.service';

import { CvResumeVersionFormGroup, CvResumeVersionFormService } from './cv-resume-version-form.service';

@Component({
  changeDetection: ChangeDetectionStrategy.OnPush,
  selector: 'jhi-cv-resume-version-update',
  templateUrl: './cv-resume-version-update.html',
  imports: [TranslateDirective, TranslatePipe, FontAwesomeModule, AlertError, ReactiveFormsModule],
})
export class CvResumeVersionUpdate implements OnInit {
  readonly isSaving = signal(false);
  cvResumeVersion: ICvResumeVersion | null = null;

  cvResumesSharedCollection = signal<ICvResume[]>([]);

  protected dataUtils = inject(DataUtils);
  protected eventManager = inject(EventManager);
  protected cvResumeVersionService = inject(CvResumeVersionService);
  protected cvResumeVersionFormService = inject(CvResumeVersionFormService);
  protected cvResumeService = inject(CvResumeService);
  protected activatedRoute = inject(ActivatedRoute);

  // eslint-disable-next-line @typescript-eslint/member-ordering
  editForm: CvResumeVersionFormGroup = this.cvResumeVersionFormService.createCvResumeVersionFormGroup();

  compareCvResume = (o1: ICvResume | null, o2: ICvResume | null): boolean => this.cvResumeService.compareCvResume(o1, o2);

  ngOnInit(): void {
    this.activatedRoute.data.subscribe(({ cvResumeVersion }) => {
      this.cvResumeVersion = cvResumeVersion;
      if (cvResumeVersion) {
        this.updateForm(cvResumeVersion);
      }

      this.loadRelationshipsOptions();
    });
  }

  byteSize(base64String: string): string {
    return this.dataUtils.byteSize(base64String);
  }

  openFile(base64String: string, contentType: string | null | undefined): void {
    this.dataUtils.openFile(base64String, contentType);
  }

  setFileData(event: Event, field: string, isImage: boolean): void {
    this.dataUtils.loadFileToForm(event, this.editForm, field, isImage).subscribe({
      error: (err: FileLoadError) =>
        this.eventManager.broadcast(
          new EventWithContent<AlertErrorModel>('jobSearchRustApp.error', { ...err, key: `error.file.${err.key}` }),
        ),
    });
  }

  previousState(): void {
    globalThis.history.back();
  }

  save(): void {
    this.isSaving.set(true);
    const cvResumeVersion = this.cvResumeVersionFormService.getCvResumeVersion(this.editForm);
    if (cvResumeVersion.id === null) {
      this.subscribeToSaveResponse(this.cvResumeVersionService.create(cvResumeVersion));
    } else {
      this.subscribeToSaveResponse(this.cvResumeVersionService.update(cvResumeVersion));
    }
  }

  protected subscribeToSaveResponse(result: Observable<ICvResumeVersion | null>): void {
    result.pipe(finalize(() => this.onSaveFinalize())).subscribe({
      next: () => this.onSaveSuccess(),
      error: () => this.onSaveError(),
    });
  }

  protected onSaveSuccess(): void {
    this.previousState();
  }

  protected onSaveError(): void {
    // Api for inheritance.
  }

  protected onSaveFinalize(): void {
    this.isSaving.set(false);
  }

  protected updateForm(cvResumeVersion: ICvResumeVersion): void {
    this.cvResumeVersion = cvResumeVersion;
    this.cvResumeVersionFormService.resetForm(this.editForm, cvResumeVersion);

    this.cvResumesSharedCollection.update(cvResumes =>
      this.cvResumeService.addCvResumeToCollectionIfMissing<ICvResume>(cvResumes, cvResumeVersion.resume),
    );
  }

  protected loadRelationshipsOptions(): void {
    this.cvResumeService
      .query()
      .pipe(map((res: HttpResponse<ICvResume[]>) => res.body ?? []))
      .pipe(
        map((cvResumes: ICvResume[]) =>
          this.cvResumeService.addCvResumeToCollectionIfMissing<ICvResume>(cvResumes, this.cvResumeVersion?.resume),
        ),
      )
      .subscribe((cvResumes: ICvResume[]) => this.cvResumesSharedCollection.set(cvResumes));
  }
}
